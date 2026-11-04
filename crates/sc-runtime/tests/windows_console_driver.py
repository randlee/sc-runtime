"""Native isolated console-event proof with the shared bounded process-tree owner."""
from __future__ import annotations

import ctypes
from ctypes import wintypes
import http.client
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[3]
WAIT = 15


def owner_module():
    path = ROOT / "tests/integration/generation/owned_process.py"
    spec = importlib.util.spec_from_file_location("runtime_console_owner", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def message(listener, expected):
    stream, _ = listener.accept()
    stream.settimeout(WAIT)
    notice = json.loads(stream.makefile("rb").readline())
    assert notice["event"] == expected, notice
    return stream, notice


def release(stream):
    stream.sendall(b"release\n")
    stream.close()


def console_sender(pid, event):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    handler_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.DWORD)
    handler = handler_type(lambda _event: True)
    kernel.FreeConsole.argtypes = []
    kernel.FreeConsole.restype = wintypes.BOOL
    kernel.AttachConsole.argtypes = [wintypes.DWORD]
    kernel.AttachConsole.restype = wintypes.BOOL
    kernel.SetConsoleCtrlHandler.argtypes = [handler_type, wintypes.BOOL]
    kernel.SetConsoleCtrlHandler.restype = wintypes.BOOL
    kernel.GenerateConsoleCtrlEvent.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.GenerateConsoleCtrlEvent.restype = wintypes.BOOL
    kernel.FreeConsole()
    if not kernel.AttachConsole(pid):
        raise ctypes.WinError(ctypes.get_last_error())
    if not kernel.SetConsoleCtrlHandler(handler, True):
        raise ctypes.WinError(ctypes.get_last_error())
    if not kernel.GenerateConsoleCtrlEvent(event, 0):
        raise ctypes.WinError(ctypes.get_last_error())
    # Keep the callback alive while this controller is attached. The signal
    # targets only the new console owned by this test child, never the runner.
    return kernel, handler


def get(endpoint, path):
    host, port = endpoint.removeprefix("tcp:").rsplit(":", 1)
    connection = http.client.HTTPConnection(host, int(port), timeout=WAIT)
    try:
        connection.request("GET", path)
        response = connection.getresponse()
        return response.status, json.loads(response.read())
    finally:
        connection.close()


def scenario(binary, directory, event, early):
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.settimeout(WAIT)
        environment = dict(os.environ, SC_RUNTIME_CONSOLE_CHILD="1",
            SC_RUNTIME_CONSOLE_ROOT=str(directory),
            SC_RUNTIME_CONSOLE_CONTROL="%s:%s" % listener.getsockname())
        if early:
            environment["SC_RUNTIME_CONSOLE_EARLY"] = "1"
        with tempfile.TemporaryFile() as output:
            child = subprocess.Popen([binary, "--exact", "console_child", "--nocapture"],
                env=environment, stdout=output, stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NEW_CONSOLE)
            attached = None
            try:
                if early:
                    entered, _ = message(listener, "stores_entered")
                    attached = console_sender(child.pid, event)
                    release(entered)
                receipt, notice = message(listener, "bound")
                receipt.close()
                endpoint = notice["endpoint"]
                response = []
                requester = None
                if not early:
                    assert get(endpoint, "/health") == (200, {"version": "1", "ok": True, "data": {"status": "ok"}, "error": None})
                    def request():
                        try:
                            response.append(get(endpoint, "/slow"))
                        except Exception as error:
                            response.append(error)
                    requester = threading.Thread(target=request)
                    requester.start()
                    inflight, _ = message(listener, "inflight")
                    attached = console_sender(child.pid, event)
                    release(inflight)
                cleanup, _ = message(listener, "cleanup")
                # A real contender must see the old lock while cleanup owns it.
                probe_env = dict(environment, SC_RUNTIME_CONSOLE_LOCK_PROBE="1")
                probe = subprocess.run([binary, "--exact", "console_child", "--nocapture"],
                    env=probe_env, capture_output=True, timeout=WAIT,
                    creationflags=subprocess.CREATE_NEW_CONSOLE)
                assert probe.returncode == 0, probe.stdout + probe.stderr
                release(cleanup)
                assert child.wait(timeout=WAIT) == 0
                if requester:
                    requester.join(timeout=WAIT)
                    assert not requester.is_alive()
                    assert response == [(200, "done")], response
                print(json.dumps({"signal":event,"early":early,"endpoint":endpoint,"cleanup":1,"exit":0}), flush=True)
            finally:
                if attached:
                    attached[0].FreeConsole()
                if child.poll() is None:
                    child.kill()
                    child.wait(timeout=WAIT)
                if child.returncode:
                    output.seek(0)
                    sys.stderr.buffer.write(output.read(65536))


def worker(binary):
    for event in (0, 1):  # real CTRL_C_EVENT and CTRL_BREAK_EVENT
        with tempfile.TemporaryDirectory(prefix="runtime-console-") as temporary:
            directory = Path(temporary)
            scenario(binary, directory, event, False)
            scenario(binary, directory, event, False)  # same-root clean restart
            scenario(binary, directory, event, True)   # signal during store setup


def main():
    assert os.name == "nt", "native Windows only"
    assert os.environ.get("SC_RUNTIME_EPHEMERAL_CI") == "1", "isolated CI only"
    if len(sys.argv) == 3 and sys.argv[1] == "--worker":
        worker(sys.argv[2])
        return
    owned = owner_module()
    build = owned.run(["cargo", "test", "-p", "sc-runtime", "--features", "test-util",
        "--test", "windows_console", "--no-run", "--message-format=json"], cwd=ROOT,
        timeout=600, capture_output=True, text=True)
    if build.returncode:
        raise RuntimeError(build.stderr)
    artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")]
    binaries = [entry["executable"] for entry in artifacts
        if entry.get("reason") == "compiler-artifact" and entry.get("target", {}).get("name") == "windows_console" and entry.get("executable")]
    assert len(binaries) == 1, binaries
    result = owned.run([sys.executable, __file__, "--worker", binaries[0]], cwd=ROOT,
        timeout=180, capture_output=True, text=True)
    print(result.stdout, end="")
    if result.returncode:
        raise RuntimeError(result.stderr)


if __name__ == "__main__":
    main()
