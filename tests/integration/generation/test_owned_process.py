"""Fault witnesses for owned test subprocess trees; no daemon is involved."""

import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock
from pathlib import Path

try:
    from tests.integration.generation import owned_process
except ModuleNotFoundError:
    import owned_process

run = owned_process.run


class OwnedProcessTests(unittest.TestCase):
    def test_empty_membership_does_not_complete_unsignaled_retained_identity(self):
        self.assertFalse(owned_process._windows_physical_complete(0, [0, 258]))
        self.assertFalse(owned_process._windows_physical_complete(1, [0]))
        self.assertTrue(owned_process._windows_physical_complete(0, [0, 0]))

    @unittest.skipIf(os.name == "nt", "Unix session inspection only")
    def test_unavailable_session_inspection_refuses_before_launch(self):
        # Mock the ps inspection separately from the target launcher Popen.
        with mock.patch.object(owned_process, "_is_nested", return_value=False), \
             mock.patch.object(owned_process.subprocess, "run", side_effect=PermissionError("ps denied")), \
             mock.patch.object(owned_process.subprocess, "Popen") as launch:
            with self.assertRaisesRegex(RuntimeError, "Unix session inspection unavailable"):
                run([sys.executable, "-c", "pass"], timeout=5)
            launch.assert_not_called()

    @unittest.skipIf(os.name == "nt", "Unix session inspection only")
    def test_unavailable_native_inspection_refuses_before_launch(self):
        listed = subprocess.CompletedProcess([], 0, stdout=f"{os.getpid()} R\n")
        with mock.patch.object(owned_process, "_is_nested", return_value=False), \
             mock.patch.object(owned_process, "_session_listing", return_value=listed), \
             mock.patch.object(owned_process, "_native_session_processes", side_effect=OSError("native unavailable")), \
             mock.patch.object(owned_process.subprocess, "Popen") as launch:
            with self.assertRaisesRegex(RuntimeError, "Unix session inspection unavailable"):
                run([sys.executable, "-c", "pass"], timeout=5)
            launch.assert_not_called()

    def test_linux_native_inspection_reads_session_and_physical_state(self):
        def entry(pid, stat):
            item = mock.MagicMock()
            item.name = str(pid)
            if isinstance(stat, Exception):
                (item / "stat").read_text.side_effect = stat
            else:
                (item / "stat").read_text.return_value = stat
            return item

        entries = [entry(123, "123 (name with ) parentheses) S 1 123 77 0"),
                   entry(124, "124 (dead) Z 1 124 77 0"),
                   entry(125, "125 (other) R 1 125 78 0"),
                   entry(126, FileNotFoundError())]
        with mock.patch.object(owned_process.sys, "platform", "linux"), \
             mock.patch.object(owned_process, "Path") as proc:
            proc.return_value.iterdir.return_value = entries
            self.assertEqual(owned_process._native_session_processes(77), [(123, "S"), (124, "Z")])

    def postlaunch_inspection_failure(self, failure):
        with tempfile.TemporaryDirectory(prefix="owned-inspection-fault-") as temp:
            root = Path(temp)
            ready, identities = root / "ready", root / "identities"
            target = root / "target.py"
            target.write_text(
                "import json,os,pathlib,subprocess,sys,time\n"
                "ready=pathlib.Path(sys.argv[1])\n"
                "code='import os,pathlib,sys,threading; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); threading.Event().wait()'\n"
                "child=subprocess.Popen([sys.executable,'-c',code,str(ready)],process_group=0)\n"
                "deadline=time.monotonic()+5\n"
                "while not ready.exists():\n"
                " if time.monotonic()>=deadline: raise RuntimeError('descendant readiness failed')\n"
                " time.sleep(.01)\n"
                "pathlib.Path(sys.argv[2]).write_text(json.dumps([child.pid,os.getsid(0)]))\n"
            )
            original = owned_process._session_listing
            calls = 0

            def inspection():
                nonlocal calls
                calls += 1
                if calls == 1:
                    return original()
                if isinstance(failure, str):
                    return subprocess.CompletedProcess([], 0, stdout=failure)
                raise failure

            try:
                with mock.patch.object(owned_process, "_is_nested", return_value=False), \
                     mock.patch.object(owned_process, "_session_listing", side_effect=inspection):
                    expected = ValueError if isinstance(failure, str) else type(failure)
                    with self.assertRaises(expected) as caught:
                        run([sys.executable, str(target), str(ready), str(identities)],
                            timeout=8, check=True, capture_output=True)
                if not isinstance(failure, str):
                    self.assertIs(caught.exception, failure, "cleanup must retain the actual inspection failure")
                self.assertEqual(calls, 2, "failed external inspection must not be retried")
                self.assertTrue(ready.exists(), "inspection fault preceded acknowledged descendant")
                descendant, _session = json.loads(identities.read_text())
                self.assert_exited(descendant)
            finally:
                # This separate native owner also reclaims the deliberately
                # broken implementation used by the negative control.
                if identities.exists():
                    _descendant, session = json.loads(identities.read_text())
                    self.reclaim_witness_session(session)

    def reclaim_witness_session(self, session):
        deadline = time.monotonic() + 10
        while True:
            active = [(pid, state) for pid, state in owned_process._native_session_processes(session)
                      if not state.startswith("Z")]
            if not active:
                return
            for pid, _state in active:
                try:
                    if os.getsid(pid) == session:
                        os.kill(pid, 9)
                except ProcessLookupError:
                    pass
            if time.monotonic() >= deadline:
                raise TimeoutError("negative witness did not physically exit")
            time.sleep(.02)

    @unittest.skipIf(os.name == "nt", "Unix session inspection only")
    def test_postlaunch_inspector_spawn_failure_reaps_other_group(self):
        self.postlaunch_inspection_failure(OSError("postlaunch ps unavailable"))

    @unittest.skipIf(os.name == "nt", "Unix session inspection only")
    def test_postlaunch_inspector_nonzero_reaps_other_group(self):
        self.postlaunch_inspection_failure(subprocess.CalledProcessError(7, ["ps"]))

    @unittest.skipIf(os.name == "nt", "Unix session inspection only")
    def test_postlaunch_inspector_timeout_reaps_other_group(self):
        self.postlaunch_inspection_failure(subprocess.TimeoutExpired(["ps"], 2))

    @unittest.skipIf(os.name == "nt", "Unix session inspection only")
    def test_postlaunch_inspector_malformed_output_reaps_other_group(self):
        self.postlaunch_inspection_failure("not-a-pid R\n")

    def assert_exited(self, pid):
        if os.name == "nt":
            import ctypes
            from ctypes import wintypes
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            kernel.OpenProcess.restype = wintypes.HANDLE
            kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
            kernel.CloseHandle.argtypes = [wintypes.HANDLE]
            handle = kernel.OpenProcess(0x1000 | 0x100000, False, pid)
            if not handle:
                return
            kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
            result = kernel.WaitForSingleObject(handle, 0)
            kernel.CloseHandle(handle)
            self.assertEqual(result, 0, f"owned process {pid} remained active when run returned")
            return
        else:
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                return
            probe = subprocess.run(["ps", "-o", "stat=", "-p", str(pid)], capture_output=True, text=True, timeout=2)
            if not probe.stdout.strip() or probe.stdout.strip().startswith("Z"):
                return
        self.fail(f"owned witness process {pid} survived cleanup")


    def witness(self, failure, new_group=False, runner=run):
        with tempfile.TemporaryDirectory(prefix="owned-witness-") as temp:
            root = Path(temp)
            ready = root / "ready"
            pidfile = root / "pids"
            grandchild = root / "grandchild.py"
            grandchild.write_text("import os, pathlib, sys, time\npathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\nwhile True: time.sleep(1)\n")
            parent = root / "parent.py"
            parent.write_text(
                "import json, os, pathlib, subprocess, sys, time\n"
                "r=pathlib.Path(sys.argv[1]); opts={'process_group':0} if sys.argv[4]=='1' and os.name!='nt' else {}; p=subprocess.Popen([sys.executable,sys.argv[2],str(r)], **opts)\n"
                "deadline=time.monotonic()+5\n"
                "while not r.exists():\n"
                " if time.monotonic()>deadline: raise RuntimeError('ready timeout')\n"
                " time.sleep(.01)\n"
                "pathlib.Path(sys.argv[3]).write_text(json.dumps([os.getpid(), p.pid]))\n"
                + ("sys.exit(0)\n" if failure == "success" else "sys.exit(7)\n" if failure else "while True: time.sleep(1)\n")
            )
            args = [sys.executable, str(parent), str(ready), str(grandchild), str(pidfile), "1" if new_group else "0"]
            if failure == "success":
                result = runner(args, timeout=8, check=True, capture_output=True)
                self.assertEqual(result.returncode, 0)
            elif failure:
                with self.assertRaises(subprocess.CalledProcessError):
                    runner(args, timeout=8, check=True, capture_output=True)
            else:
                with self.assertRaises(subprocess.TimeoutExpired):
                    runner(args, timeout=3, check=True, capture_output=True)
            self.assertTrue(ready.exists(), "grandchild never acknowledged readiness")
            direct, descendant = json.loads(pidfile.read_text())
            self.assert_exited(direct)
            self.assert_exited(descendant)

    def test_success_cleans_ready_descendant(self):
        self.witness("success")

    def test_timeout_cleans_ready_descendant(self):
        self.witness(False)

    def test_failure_cleans_ready_descendant(self):
        self.witness(True)

    def test_timeout_cleans_descendant_in_new_process_group(self):
        self.witness(False, new_group=True)

    def test_failure_cleans_descendant_in_new_process_group(self):
        self.witness(True, new_group=True)

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

    @unittest.skipIf(os.name == "nt", "Unix session identity only")
    def test_nested_detection_uses_session_in_distinct_process_group(self):
        with mock.patch.dict(os.environ, {owned_process._MARKER: "71"}), \
             mock.patch.object(owned_process.os, "getsid", return_value=71), \
             mock.patch.object(owned_process.os, "getpgrp", return_value=99):
            self.assertTrue(owned_process._is_nested())

    @unittest.skipIf(os.name == "nt", "Unix session identity only")
    def test_nested_other_group_keeps_owned_session_and_reaps_descendant(self):
        with tempfile.TemporaryDirectory(prefix="owned-nested-other-group-") as temp:
            root = Path(temp)
            ready, expected = root / "ready", root / "expected-session"
            target = root / "target.py"
            target.write_text(
                "import os,pathlib,subprocess,sys,time\n"
                "ready=pathlib.Path(sys.argv[1])\n"
                "code='import json,os,pathlib,sys,threading; pathlib.Path(sys.argv[1]).write_text(json.dumps([os.getpid(),os.getsid(0)])); threading.Event().wait()'\n"
                "child=subprocess.Popen([sys.executable,'-c',code,str(ready)],process_group=0)\n"
                "deadline=time.monotonic()+5\n"
                "while not ready.exists():\n"
                " if time.monotonic()>=deadline: raise RuntimeError('nested descendant readiness failed')\n"
                " time.sleep(.01)\n"
                "sys.exit(7)\n"
            )
            code = (
                "import os,pathlib,sys; pathlib.Path(sys.argv[3]).write_text(str(os.getsid(0))); "
                "os.setpgid(0,0); from tests.integration.generation.owned_process import run; "
                "run([sys.executable,sys.argv[1],sys.argv[2]],timeout=8,check=True)"
            )
            try:
                with self.assertRaises(subprocess.CalledProcessError):
                    run([sys.executable, "-c", code, str(target), str(ready), str(expected)],
                        cwd=Path(__file__).resolve().parents[3], timeout=10,
                        check=True, capture_output=True)
                self.assertTrue(ready.exists(), "nested descendant never acknowledged readiness")
                descendant, session = json.loads(ready.read_text())
                self.assert_exited(descendant)
                self.assertEqual(session, int(expected.read_text()), "nested helper escaped its owned session")
            finally:
                if ready.exists():
                    _descendant, session = json.loads(ready.read_text())
                    self.reclaim_witness_session(session)

    def test_success_preserves_output(self):
        result = run([sys.executable, "-c", "print('ok')"], timeout=5, check=True, capture_output=True, text=True)
        self.assertEqual(result.stdout.strip(), "ok")


if __name__ == "__main__":
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(OwnedProcessTests)
    if suite.countTestCases() < 7:
        raise SystemExit("owned-process witness suite is incomplete")
    sys.exit(0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1)
