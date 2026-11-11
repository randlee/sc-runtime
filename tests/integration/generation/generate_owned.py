"""Test-owned command boundary for cargo-generate and its hooks."""

import importlib.util
from pathlib import Path
import sys

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[3]))

from tests.integration.generation.owned_process import run


def run_generation(command, *, timeout=120, **kwargs):
    """Run tools in the canonical owned tree with file-backed capture."""
    kwargs.setdefault("text", True)
    kwargs.setdefault("capture_output", True)
    return run(command, timeout=timeout, **kwargs)


def cargo_generate(root, destination, name, values, *, env=None, timeout=120):
    """Bound the entire production driver call, including version checks/hooks."""
    return run_generation(
        [sys.executable, str(Path(__file__).resolve()), "--generate", str(root),
         str(destination), name, str(values)],
        timeout=timeout, env=env, check=True,
    )


def main(argv):
    if len(argv) != 5 or argv[0] != "--generate":
        return 2
    root, destination, name, values = argv[1:]
    spec = importlib.util.spec_from_file_location(
        "owned_generation_driver", Path(__file__).resolve().parents[3] / "scripts/new_project.py")
    driver = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(driver)
    driver.cargo_generate(Path(root), Path(destination), name, Path(values))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
