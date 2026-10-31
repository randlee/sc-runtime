"""Fault witnesses for owned test subprocess trees; no daemon is involved."""

import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from tests.integration.generation.owned_process import run


class OwnedProcessTests(unittest.TestCase):
    def assert_exited(self, pid):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if os.name == "nt":
                import ctypes
                from ctypes import wintypes
                kernel = ctypes.WinDLL("kernel32", use_last_error=True)
                kernel.OpenProcess.restype = wintypes.HANDLE
                kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
                kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
                kernel.CloseHandle.argtypes = [wintypes.HANDLE]
                handle = kernel.OpenProcess(0x1000, False, pid)
                if not handle:
                    return
                code = wintypes.DWORD()
                ok = kernel.GetExitCodeProcess(handle, ctypes.byref(code))
                kernel.CloseHandle(handle)
                if ok and code.value != 259:
                    return
            else:
                try:
                    os.kill(pid, 0)
                except ProcessLookupError:
                    return
                probe = subprocess.run(["ps", "-o", "stat=", "-p", str(pid)], capture_output=True, text=True, timeout=2)
                if not probe.stdout.strip() or probe.stdout.strip().startswith("Z"):
                    return
            time.sleep(0.02)
        self.fail(f"owned witness process {pid} survived cleanup")

    def witness(self, failure):
        with tempfile.TemporaryDirectory(prefix="owned-witness-") as temp:
            root = Path(temp)
            ready = root / "ready"
            pidfile = root / "pids"
            grandchild = root / "grandchild.py"
            grandchild.write_text("import os, pathlib, sys, time\npathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\nwhile True: time.sleep(1)\n")
            parent = root / "parent.py"
            parent.write_text(
                "import json, os, pathlib, subprocess, sys, time\n"
                "r=pathlib.Path(sys.argv[1]); p=subprocess.Popen([sys.executable,sys.argv[2],str(r)])\n"
                "deadline=time.monotonic()+5\n"
                "while not r.exists():\n"
                " if time.monotonic()>deadline: raise RuntimeError('ready timeout')\n"
                " time.sleep(.01)\n"
                "pathlib.Path(sys.argv[3]).write_text(json.dumps([os.getpid(), p.pid]))\n"
                + ("sys.exit(7)\n" if failure else "while True: time.sleep(1)\n")
            )
            args = [sys.executable, str(parent), str(ready), str(grandchild), str(pidfile)]
            if failure:
                with self.assertRaises(subprocess.CalledProcessError):
                    run(args, timeout=8, check=True, capture_output=True)
            else:
                with self.assertRaises(subprocess.TimeoutExpired):
                    run(args, timeout=3, check=True, capture_output=True)
            self.assertTrue(ready.exists(), "grandchild never acknowledged readiness")
            direct, descendant = json.loads(pidfile.read_text())
            self.assert_exited(direct)
            self.assert_exited(descendant)

    def test_timeout_cleans_ready_descendant(self):
        self.witness(False)

    def test_failure_cleans_ready_descendant(self):
        self.witness(True)

    def test_nested_timeout_aborts_enclosing_tree(self):
        with tempfile.TemporaryDirectory(prefix="owned-nested-") as temp:
            root = Path(temp)
            ready = root / "ready"
            witness = root / "witness.py"
            witness.write_text(
                "import os, pathlib, sys, time\n"
                "pathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\n"
                "while True: time.sleep(1)\n"
            )
            code = (
                "import sys; from tests.integration.generation.owned_process import run; "
                "run([sys.executable, sys.argv[1], sys.argv[2]], timeout=1)"
            )
            with self.assertRaises(subprocess.CalledProcessError):
                run([sys.executable, "-c", code, str(witness), str(ready)],
                    cwd=Path(__file__).resolve().parents[3], timeout=8, check=True, capture_output=True)
            self.assertTrue(ready.exists(), "nested child never acknowledged readiness")
            self.assert_exited(int(ready.read_text()))

    def test_nested_failure_aborts_ready_descendant(self):
        with tempfile.TemporaryDirectory(prefix="owned-nested-failure-") as temp:
            root = Path(temp)
            ready = root / "ready"
            witness = root / "witness.py"
            witness.write_text(
                "import os, pathlib, subprocess, sys, time\n"
                "p=subprocess.Popen([sys.executable, '-c', "
                "'import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); time.sleep(60)', sys.argv[1]])\n"
                "r=pathlib.Path(sys.argv[1]); deadline=time.monotonic()+5\n"
                "while not r.exists():\n"
                " if time.monotonic()>deadline: raise RuntimeError('ready timeout')\n"
                " time.sleep(.01)\n"
                "sys.exit(7)\n"
            )
            code = ("import sys; from tests.integration.generation.owned_process import run; "
                    "run([sys.executable, sys.argv[1], sys.argv[2]], timeout=8, check=True)")
            with self.assertRaises(subprocess.CalledProcessError):
                run([sys.executable, "-c", code, str(witness), str(ready)],
                    cwd=Path(__file__).resolve().parents[3], timeout=10, check=True, capture_output=True)
            self.assertTrue(ready.exists(), "nested grandchild never acknowledged readiness")
            self.assert_exited(int(ready.read_text()))

    def test_success_preserves_output(self):
        result = run([sys.executable, "-c", "print('ok')"], timeout=5, check=True, capture_output=True, text=True)
        self.assertEqual(result.stdout.strip(), "ok")


if __name__ == "__main__":
    unittest.main()
