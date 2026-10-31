"""Bounded ownership for test command trees, including nested recipe wrappers."""

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path


_MARKER = "SC_RUNTIME_TEST_OWNED_SESSION"


def _windows_job(process):
    import ctypes
    from ctypes import wintypes

    class Basic(ctypes.Structure):
        _fields_ = [("PerProcessUserTimeLimit", ctypes.c_longlong), ("PerJobUserTimeLimit", ctypes.c_longlong),
                    ("LimitFlags", wintypes.DWORD), ("MinimumWorkingSetSize", ctypes.c_size_t),
                    ("MaximumWorkingSetSize", ctypes.c_size_t), ("ActiveProcessLimit", wintypes.DWORD),
                    ("Affinity", ctypes.c_size_t), ("PriorityClass", wintypes.DWORD), ("SchedulingClass", wintypes.DWORD)]

    class Io(ctypes.Structure):
        _fields_ = [(name, ctypes.c_ulonglong) for name in ("ReadOperationCount", "WriteOperationCount", "OtherOperationCount", "ReadTransferCount", "WriteTransferCount", "OtherTransferCount")]

    class Extended(ctypes.Structure):
        _fields_ = [("BasicLimitInformation", Basic), ("IoInfo", Io), ("ProcessMemoryLimit", ctypes.c_size_t),
                    ("JobMemoryLimit", ctypes.c_size_t), ("PeakProcessMemoryUsed", ctypes.c_size_t), ("PeakJobMemoryUsed", ctypes.c_size_t)]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateJobObjectW.restype = wintypes.HANDLE
    kernel.CreateJobObjectW.argtypes = [ctypes.c_void_p, wintypes.LPCWSTR]
    kernel.SetInformationJobObject.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    kernel.AssignProcessToJobObject.argtypes = [wintypes.HANDLE, wintypes.HANDLE]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    job = kernel.CreateJobObjectW(None, None)
    if not job:
        raise ctypes.WinError(ctypes.get_last_error())
    limits = Extended()
    limits.BasicLimitInformation.LimitFlags = 0x2000  # KILL_ON_JOB_CLOSE
    if not kernel.SetInformationJobObject(job, 9, ctypes.byref(limits), ctypes.sizeof(limits)) or not kernel.AssignProcessToJobObject(job, wintypes.HANDLE(process._handle)):
        error = ctypes.get_last_error()
        kernel.CloseHandle(job)
        raise ctypes.WinError(error)
    return kernel, job


def run(args, *, timeout, cwd=None, env=None, stdin=None, text=False, capture_output=False, check=False):
    """Run a command in an owned tree; timeout and failure clean up descendants."""
    nested = os.environ.get(_MARKER) == "1"
    with tempfile.TemporaryDirectory(prefix="sc-test-owned-") as temp:
        root = Path(temp)
        config = root / "command.json"
        start = root / "start"
        status = root / "status"
        config.write_text(json.dumps({"args": list(map(str, args)), "start": str(start), "status": str(status)}))
        child_env = dict(os.environ if env is None else env)
        child_env[_MARKER] = "1"
        with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
            child = subprocess.Popen(
                [sys.executable, str(Path(__file__).resolve()), "--launch", str(config)], cwd=cwd,
                env=child_env, stdin=stdin, stdout=out if capture_output else None,
                stderr=err if capture_output else None, start_new_session=(os.name != "nt" and not nested),
            )
            job = None
            timed_out = False
            result = None
            try:
                if os.name == "nt":
                    job = _windows_job(child)
                start.touch()
                deadline = time.monotonic() + timeout
                while not status.exists():
                    if time.monotonic() >= deadline:
                        timed_out = True
                        break
                    time.sleep(0.02)
                if not timed_out:
                    result = json.loads(status.read_text())["returncode"]
            finally:
                # The launcher stays alive until cleanup, pinning the process-group ID.
                if job is not None:
                    job[0].CloseHandle(job[1])
                elif not nested and os.name != "nt":
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                else:
                    try:
                        child.kill()
                    except ProcessLookupError:
                        pass
                child.wait(timeout=10)
                if nested and os.name != "nt" and (timed_out or result != 0):
                    # Abort the enclosing owned tree as well: descendants may
                    # still hold the driver's captured output pipes open.
                    os.killpg(os.getpgrp(), signal.SIGTERM)
            out.seek(0)
            err.seek(0)
            stdout = out.read(32 * 1024 * 1024 + 1) if capture_output else None
            stderr = err.read(65536) if capture_output else None
            if stdout is not None and len(stdout) > 32 * 1024 * 1024:
                raise RuntimeError("owned command stdout exceeded 32 MiB")
            if text:
                stdout = stdout.decode(errors="replace") if stdout is not None else None
                stderr = stderr.decode(errors="replace") if stderr is not None else None
            if timed_out:
                raise subprocess.TimeoutExpired(args, timeout, output=stdout, stderr=stderr)
            completed = subprocess.CompletedProcess(args, result, stdout, stderr)
            if check:
                completed.check_returncode()
            return completed


def _launch(config_path):
    config = json.loads(Path(config_path).read_text())
    start, status = Path(config["start"]), Path(config["status"])
    deadline = time.monotonic() + 30
    while not start.exists():
        if time.monotonic() >= deadline:
            return 1
        time.sleep(0.02)
    if os.name != "nt":
        def abort(_signum, _frame):
            temporary = status.with_suffix(".tmp")
            temporary.write_text(json.dumps({"returncode": 124}))
            os.replace(temporary, status)
            while True:
                time.sleep(1)
        signal.signal(signal.SIGTERM, abort)
    try:
        result_code = subprocess.run(config["args"], check=False).returncode
    except OSError as error:
        print(f"owned command failed to start: {error}", file=sys.stderr)
        result_code = 127
    temporary = status.with_suffix(".tmp")
    temporary.write_text(json.dumps({"returncode": result_code}))
    os.replace(temporary, status)
    while True:
        time.sleep(1)


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--launch":
        sys.exit(_launch(sys.argv[2]))
    sys.exit(2)
