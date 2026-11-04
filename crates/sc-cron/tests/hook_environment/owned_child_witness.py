"""Causal Windows-only witnesses for the actual Rust bridge entry point."""
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace


def publish(path, value):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value))
    os.replace(temporary, path)


def until(path):
    deadline = time.monotonic() + 5
    while not path.exists():
        if time.monotonic() >= deadline:
            raise TimeoutError(f'witness readiness missing: {path.name}')
        time.sleep(.01)


def target(root, mode):
    descendant = subprocess.Popen([sys.executable, __file__, 'descendant', str(root)])
    until(root / 'descendant-ready')
    publish(root / 'pids', [os.getpid(), descendant.pid])
    until(root / 'release')
    if mode == 'timeout':
        while True:
            time.sleep(1)


def bridge(mode, root, request):
    publish(root / 'bridge-pid', os.getpid())
    if mode in ('stall', 'omit'):
        # Fault fixture deliberately bypasses the inner owner. Rust must own it.
        if mode == 'stall':
            descendant = subprocess.Popen([sys.executable, __file__, 'descendant', str(root)])
            until(root / 'descendant-ready')
            publish(root / 'pids', [os.getpid(), descendant.pid])
            while True:
                time.sleep(1)
        child = subprocess.Popen([sys.executable, __file__, 'target', str(root), 'success'])
        until(root / 'pids')
        until(root / 'release')
        child.wait(timeout=5)
        return 0
    sys.path.insert(0, str(Path(__file__).resolve().parents[4]))
    from tests.integration.generation import owned_process
    import owned_child
    if mode == 'timeout':
        # Freeze only the owner module's clock, with independent real-clock
        # startup bounds above and in Rust. Expire after controller release.
        real_sleep, real_now = time.sleep, time.monotonic
        expired_at = None

        def clock():
            nonlocal expired_at
            if expired_at is None and (root / 'expire').exists():
                expired_at = real_now()
            # After injection the clock advances normally, so cleanup bounds
            # remain real bounds rather than freezing an unreapable job.
            return 0 if expired_at is None else 1000 + real_now() - expired_at

        owned_process.time = SimpleNamespace(monotonic=clock, sleep=real_sleep)
        config = json.loads(Path(request).read_text())
        config['timeout'] = 3  # Original execution bound, never startup padding.
        request = root / 'inner-request'
        request.write_text(json.dumps(config))
    return owned_child.main(request)


def observe(root):
    until(root / 'pids')
    until(root / 'bridge-pid')
    pids = json.loads((root / 'pids').read_text()) + [json.loads((root / 'bridge-pid').read_text())]
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handles = []
    try:
        for pid in pids:
            handle = kernel.OpenProcess(0x100000, False, pid)  # SYNCHRONIZE
            if not handle:
                raise ctypes.WinError(ctypes.get_last_error())
            handles.append(handle)
        publish(root / 'observer-ready', True)
        for phase in ('before', 'after'):
            until(root / phase)
            states = []
            for handle in handles:
                result = kernel.WaitForSingleObject(handle, 0)
                if result not in (0, 258):
                    raise ctypes.WinError(ctypes.get_last_error())
                states.append(result == 0)
            publish(root / (phase + '-result'), states)
    finally:
        for handle in handles:
            kernel.CloseHandle(handle)


if __name__ == '__main__':
    action, root = sys.argv[1], Path(sys.argv[2])
    if action == 'descendant':
        publish(root / 'descendant-ready', os.getpid())
        while True:
            time.sleep(1)
    elif action == 'target':
        target(root, sys.argv[3])
    elif action == 'observe':
        observe(root)
