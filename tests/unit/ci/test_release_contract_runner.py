"""Cancellation reaps a ready installer tree before deleting its environment."""

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[3]


class ReleaseContractRunnerTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "posix", "SIGTERM ownership witness uses Unix sessions")
    def test_ready_hanging_installer_is_stopped_before_environment_cleanup(self):
        with tempfile.TemporaryDirectory(prefix="release-contract-witness-") as directory:
            root = Path(directory)
            ready, cleanup = root / "ready.json", root / "cleanup.json"
            child = root / "child.py"
            child.write_text(
                "import json, os, pathlib, sys, time\n"
                "pathlib.Path(sys.argv[1]).write_text(json.dumps({'pids':[int(sys.argv[2]),os.getpid()],'sid':os.getsid(0)}))\n"
                "while True: time.sleep(1)\n"
            )
            installer = root / "installer.py"
            installer.write_text(
                f"#!{sys.executable}\n"
                "import os, pathlib, subprocess, sys, time\n"
                f"child={str(child)!r}; ready={str(ready)!r}\n"
                "subprocess.Popen([sys.executable, child, ready, str(os.getpid())], process_group=0)\n"
                "while True: time.sleep(1)\n"
            )
            creator = root / "creator.py"
            creator.write_text(
                f"#!{sys.executable}\n"
                "import pathlib, shutil, sys\n"
                "destination=pathlib.Path(sys.argv[-1]); (destination/'bin').mkdir(parents=True)\n"
                f"shutil.copyfile({str(installer)!r}, destination/'bin/python')\n"
                "(destination/'bin/python').chmod(0o755)\n"
            )
            creator.chmod(0o755)
            # Observe cleanup ordering in the real controller process, without
            # replacing the helper or its process/session cleanup machinery.
            controller = f"""
import importlib.util, json, pathlib, subprocess, sys
spec=importlib.util.spec_from_file_location('contracts', {str(ROOT / 'scripts/check_release_contract_sources.py')!r})
module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
module.PYTHON={str(creator)!r}
original=module.tempfile.TemporaryDirectory
class ObservedDirectory(original):
    def cleanup(self):
        active=[]
        ready=pathlib.Path({str(ready)!r})
        if ready.exists():
            for pid in json.loads(ready.read_text())['pids']:
                probe=subprocess.run(['ps','-o','stat=','-p',str(pid)], capture_output=True, text=True, timeout=2)
                if probe.stdout.strip() and not probe.stdout.strip().startswith('Z'):
                    active.append(pid)
        pathlib.Path({str(cleanup)!r}).write_text(json.dumps({{'active_before_venv_removal':active, 'environment':self.name}}))
        super().cleanup()
module.tempfile.TemporaryDirectory=ObservedDirectory
sys.exit(module.main())
"""
            env = {**os.environ, "RUNNER_TEMP": str(root), "PYTHONDONTWRITEBYTECODE": "1"}
            env.pop("SC_RUNTIME_TEST_OWNED_SESSION", None)
            process = subprocess.Popen(
                [sys.executable, "-c", controller], cwd=ROOT, env=env,
                stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, start_new_session=True,
            )
            try:
                deadline = time.monotonic() + 10
                while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(ready.exists(), "installer descendant never acknowledged readiness")
                self.assertIsNone(process.poll(), "controller exited before cancellation")
                self.assertEqual(len(list(root.glob("sc-release-contracts-*"))), 1)
                process.send_signal(signal.SIGTERM)
                _, errors = process.communicate(timeout=15)
                self.assertEqual(process.returncode, 128 + signal.SIGTERM, errors.decode())
                observed = json.loads(cleanup.read_text())
                self.assertEqual(observed["active_before_venv_removal"], [])
                self.assertFalse(Path(observed["environment"]).exists())
                self.assertFalse(list(root.glob("sc-release-contracts-*")))
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)
                if ready.exists():
                    witness = json.loads(ready.read_text())
                    for pid in witness["pids"]:
                        try:
                            if os.getsid(pid) == witness["sid"]:
                                os.kill(pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                process.stderr.close()
