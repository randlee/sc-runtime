#!/usr/bin/env python3
"""Generate one project from a validated answers file.

``--var-file`` never reads stdin, never prompts, and never starts a wizard.
Stderr is one line: ``<code> <cause> — <recovery>``.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# cargo generate takes this key through --name, not a placeholder.
PROJECT_NAME_KEY = "project_name"
CARGO_GENERATE_VERSION = "0.25.1"
JSONSCHEMA_VERSION = "4.26.0"

SCHEMA_NAME = "wizard/answers.schema.json"
RECIPES = ("setup", "lint", "test", "build-cli", "build-daemon")
STEP_RECOVERY = "fix it or remove the partial destination; later steps do not run"
COMPOSE_DOCS = ("AGENTS.md.j2", "CLAUDE.md.j2")

INVENTORY = {
    "GEN.SCHEMA_INVALID": (
        "answers fail wizard/answers.schema.json",
        "fix the file; cargo is not called; the destination is not created",
    ),
    "APP.INVALID_NAME": (
        "name fails the project-name pattern in wizard/answers.schema.json",
        "fix the name to that pattern; the destination is not created",
    ),
    "GEN.TOOL_MISSING": (
        "SC_COMPOSE unset and sc-compose not on PATH",
        "set SC_COMPOSE or install sc-compose on PATH; cargo generate does not run",
    ),
    "GEN.DEVELOPMENT_RELEASE_CONFLICT": (
        "development observability cannot be combined with release generation",
        "omit --development-observability for a registry release render",
    ),
    "GEN.DEST_CONFLICT": (
        "the destination path exists",
        "choose an absent path",
    ),
}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def fail(code: str, cause: str | None = None, recovery: str | None = None) -> None:
    if cause is None or recovery is None:
        cause, recovery = INVENTORY[code]
    sys.stderr.write(f"{code} {cause} — {recovery}\n")
    raise SystemExit(1)


def fail_step(step: str) -> None:
    fail("GEN.STEP_FAILED", f"step is {step}", STEP_RECOVERY)


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        fail("GEN.SCHEMA_INVALID")


def load_schema(root: Path) -> dict:
    schema = load_json(root / SCHEMA_NAME)
    if not isinstance(schema, dict):
        fail("GEN.SCHEMA_INVALID")
    return schema


def name_pattern(schema: dict) -> re.Pattern[str]:
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        fail("GEN.SCHEMA_INVALID")
    field = properties.get(PROJECT_NAME_KEY)
    if not isinstance(field, dict):
        fail("GEN.SCHEMA_INVALID")
    pattern = field.get("pattern")
    if not isinstance(pattern, str):
        fail("GEN.SCHEMA_INVALID")
    return re.compile(pattern)


def apply_defaults(schema: dict, answers: dict) -> dict:
    filled = dict(answers)
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        return filled
    for name, spec in properties.items():
        if name in filled or not isinstance(spec, dict):
            continue
        if "default" in spec:
            filled[name] = spec["default"]
    return filled


def schema_accepts(schema: dict, answers: object) -> bool:
    try:
        import jsonschema
    except ImportError:
        fail(
            "GEN.TOOL_MISSING",
            "jsonschema is not installed",
            f"pip install jsonschema=={JSONSCHEMA_VERSION}; cargo generate does not run",
        )
    try:
        jsonschema.Draft202012Validator(schema).validate(answers)
    except jsonschema.ValidationError:
        return False
    except jsonschema.SchemaError:
        return False
    return True


def validate(schema: dict, answers: object) -> dict:
    if not isinstance(answers, dict):
        fail("GEN.SCHEMA_INVALID")
    name = answers.get(PROJECT_NAME_KEY)
    if isinstance(name, str) and name_pattern(schema).fullmatch(name) is None:
        fail("APP.INVALID_NAME")
    if not schema_accepts(schema, answers):
        fail("GEN.SCHEMA_INVALID")
    return apply_defaults(schema, answers)


def locate_sc_compose() -> str | None:
    override = os.environ.get("SC_COMPOSE")
    if override:
        if Path(override).is_file() and os.access(override, os.X_OK):
            return override
        fail(
            "GEN.TOOL_MISSING",
            "SC_COMPOSE does not point at an executable file",
            "set SC_COMPOSE to the sc-compose binary; cargo generate does not run",
        )
    return shutil.which("sc-compose")


def require_on_path(name: str) -> None:
    if shutil.which(name) is None:
        fail(
            "GEN.TOOL_MISSING",
            f"{name} is not on PATH",
            f"install {name} on PATH; later steps do not run",
        )


def require_cargo_generate(root: Path) -> None:
    require_on_path("cargo")
    try:
        completed = subprocess.run(
            ["cargo", "generate", "--version"],
            cwd=root,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except OSError as err:
        fail(
            "GEN.TOOL_MISSING",
            f"cargo generate could not start: {err.strerror or err}",
            f"install cargo-generate {CARGO_GENERATE_VERSION}; cargo generate does not run",
        )
    text = f"{completed.stdout}\n{completed.stderr}"
    if completed.returncode != 0 or CARGO_GENERATE_VERSION not in text.split():
        fail(
            "GEN.TOOL_MISSING",
            f"cargo-generate is not {CARGO_GENERATE_VERSION}",
            f"install cargo-generate {CARGO_GENERATE_VERSION}; cargo generate does not run",
        )


def toml_scalar(value: object) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(value, list):
        return "[" + ", ".join(toml_scalar(item) for item in value) + "]"
    fail_step("values")


def write_values(answers: dict) -> Path:
    lines = ["[values]"]
    for key, value in answers.items():
        if key == PROJECT_NAME_KEY:
            continue
        lines.append(f"{key} = {toml_scalar(value)}")
    text = "\n".join(lines) + "\n"
    try:
        values_dir = os.environ.get("SC_RUNTIME_VALUES_DIR")
        handle = tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="sc-runtime-values-",
            suffix=".toml",
            dir=values_dir,
            delete=False,
        )
        with handle:
            handle.write(text)
        return Path(handle.name)
    except OSError:
        fail_step("values")


def emit_child_output(completed: subprocess.CompletedProcess[str]) -> None:
    for stream in (completed.stdout, completed.stderr):
        if not stream:
            continue
        sys.stdout.write(stream)
        if not stream.endswith("\n"):
            sys.stdout.write("\n")


def run_command(argv: list[str], cwd: Path, step: str) -> None:
    try:
        completed = subprocess.run(
            argv,
            cwd=cwd,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        fail(
            "GEN.TOOL_MISSING",
            f"{argv[0]} is not on PATH",
            f"install {argv[0]} on PATH; later steps do not run",
        )
    except OSError:
        fail_step(step)
    if completed.returncode != 0:
        emit_child_output(completed)
        fail_step(step)


def cargo_generate(root: Path, dest: Path, name: str, values: Path) -> None:
    require_cargo_generate(root)
    try:
        dest.mkdir()
    except OSError:
        fail_step("cargo-generate")
    run_command(
        [
            "cargo",
            "generate",
            "--path",
            str(root / "template"),
            "--template-values-file",
            str(values),
            "--name",
            name,
            "--destination",
            str(dest),
            "--init",
            "--silent",
            "--allow-commands",
            "--force",
            "--no-workspace",
        ],
        root,
        "cargo-generate",
    )
    # Liquid tag lines leave a blank line. Format before the recipes so
    # `just lint` sees a rustfmt-clean tree for both mcp values.
    run_command(["cargo", "fmt", "--all"], dest, "cargo-generate")


def patch_crate_names(root: Path) -> list[str]:
    text = (root / "template" / "Cargo.toml").read_text(encoding="utf-8")
    in_section = False
    names: list[str] = []
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_section = stripped == "[workspace.dependencies]"
            continue
        if not in_section or not stripped or stripped.startswith("#"):
            continue
        name = stripped.split("=", 1)[0].strip()
        if name.startswith("sc-") and (root / "crates" / name / "Cargo.toml").is_file():
            names.append(name)
    if not names:
        fail_step("cargo-generate")
    return names


def release_mode(explicit: bool) -> bool:
    if explicit:
        return True
    return os.environ.get("SC_RUNTIME_RELEASE") == "1"


def apply_checkout_patch(dest: Path, root: Path) -> None:
    manifest = dest / "Cargo.toml"
    try:
        text = manifest.read_text(encoding="utf-8")
    except OSError:
        fail_step("cargo-generate")
    if "[patch.crates-io]" in text:
        return
    lines = ["", "[patch.crates-io]"]
    for name in patch_crate_names(root):
        crate = (root / "crates" / name).resolve()
        if not (crate / "Cargo.toml").is_file():
            fail_step("cargo-generate")
        lines.append(f'{name} = {{ path = "{crate.as_posix()}" }}')
    try:
        manifest.write_text(text.rstrip() + "\n" + "\n".join(lines) + "\n", encoding="utf-8")
    except OSError:
        fail_step("cargo-generate")


def apply_development_observability(dest: Path, root: Path) -> None:
    """Select real 1.5.0 logger plus unpublished interface bridge at generation time.

    This explicit development mode never changes the canonical template or
    release artifacts. The application still owns the real logger lifecycle.
    """
    manifest = dest / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")
    types = (root / "dev/observability-types-shim").resolve().as_posix()
    adapter = (root / "dev/observability-logger-adapter").resolve().as_posix()
    text = text.replace('sc-observability = { version = "1.5.1" }',
                        'sc-observability = { version = "=1.5.0" }')
    text = text.replace('sc-observability-types = { version = "1.5.1" }',
                        f'sc-observability-types = {{ version = "1.5.1", path = "{types}" }}\n'
                        f'sc-observability-dev-adapter = {{ version = "0.0.0", path = "{adapter}" }}')
    manifest.write_text(text, encoding="utf-8")


def render_documents(dest: Path, compose: str, answers_path: Path) -> None:
    for name in COMPOSE_DOCS:
        source = dest / name
        output = dest / name.removesuffix(".j2")
        run_command(
            [
                compose,
                "render",
                "--mode",
                "file",
                "--file",
                str(source),
                "--root",
                str(dest),
                "--var-file",
                str(answers_path),
                "--output",
                str(output),
                "--unknown-var-mode",
                "ignore",
            ],
            dest,
            "sc-compose",
        )
        try:
            source.unlink()
        except OSError:
            fail_step("sc-compose")


def run_recipes(dest: Path) -> None:
    require_on_path("just")
    for step in RECIPES:
        run_command(["just", step], dest, step)


def write_answers_for_compose(answers: dict) -> Path:
    try:
        handle = tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="sc-runtime-answers-",
            suffix=".json",
            delete=False,
        )
        with handle:
            json.dump(answers, handle)
            handle.write("\n")
        return Path(handle.name)
    except OSError:
        fail_step("values")


def generate(dest: Path, var_file: Path, release: bool, development_observability: bool = False) -> None:
    if development_observability and release_mode(release):
        fail("GEN.DEVELOPMENT_RELEASE_CONFLICT")
    dest = dest.resolve()
    root = repo_root()
    schema = load_schema(root)
    answers = validate(schema, load_json(var_file))
    if dest.exists():
        fail("GEN.DEST_CONFLICT")
    compose = locate_sc_compose()
    if compose is None:
        fail("GEN.TOOL_MISSING")
    require_on_path("just")
    name = answers[PROJECT_NAME_KEY]
    if not isinstance(name, str):
        fail("GEN.SCHEMA_INVALID")
    values = write_values(answers)
    composed = write_answers_for_compose(answers)
    try:
        cargo_generate(root, dest, name, values)
        if not release_mode(release):
            apply_checkout_patch(dest, root)
        if development_observability:
            apply_development_observability(dest, root)
        render_documents(dest, compose, composed)
        run_recipes(dest)
    finally:
        values.unlink(missing_ok=True)
        composed.unlink(missing_ok=True)


def entry_answers(prefill: Path | None) -> Path:
    """Run the wizard runner and return the answers file it wrote.

    A non-zero status means the runner stopped. The destination is not created.
    The runner's last stderr line is an inventory row. This function forwards
    that stderr. An uncoded final line is replaced by ``GEN.SCHEMA_INVALID``.
    """
    handle = tempfile.NamedTemporaryFile(
        mode="w",
        encoding="utf-8",
        prefix="sc-runtime-wizard-answers-",
        suffix=".json",
        delete=False,
    )
    handle.close()
    path = Path(handle.name)
    command = [
        sys.executable,
        str(repo_root() / "scripts" / "run_wizard.py"),
        "--output",
        str(path),
    ]
    if prefill is not None:
        command.extend(["--prefill", str(prefill.resolve())])
    completed = subprocess.run(
        command,
        cwd=repo_root(),
        check=False,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    text = completed.stderr or ""
    if completed.returncode == 0:
        if text:
            sys.stderr.write(text)
        return path
    path.unlink(missing_ok=True)
    lines = [line for line in text.splitlines() if line.strip()]
    code = lines[-1].split(" ", 1)[0] if lines else ""
    known = {
        "GEN.SCHEMA_INVALID",
        "APP.INVALID_NAME",
        "GEN.TOOL_MISSING",
        "GEN.DEST_CONFLICT",
        "GEN.WIZARD_CANCELLED",
        "GEN.STEP_FAILED",
    }
    if text and not text.endswith("\n"):
        text += "\n"
    if text:
        sys.stderr.write(text)
    if code not in known:
        fail("GEN.SCHEMA_INVALID")
    raise SystemExit(completed.returncode)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("dest", nargs="?", type=Path)
    parser.add_argument("--var-file", type=Path)
    parser.add_argument("--prefill", type=Path)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--development-observability", action="store_true", help="generate against the real 1.5.0 logger and local interface bridge; incompatible with --release")
    parser.add_argument("--patch-checkout", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.development_observability and release_mode(args.release):
        fail("GEN.DEVELOPMENT_RELEASE_CONFLICT")
    if args.patch_checkout is not None:
        apply_checkout_patch(args.patch_checkout, repo_root())
        if args.development_observability:
            apply_development_observability(args.patch_checkout, repo_root())
        return
    if args.dest is None:
        fail("GEN.SCHEMA_INVALID")
    acquired: Path | None = None
    try:
        var_file = args.var_file
        if var_file is None:
            if args.dest.exists():
                fail("GEN.DEST_CONFLICT")
            acquired = entry_answers(args.prefill)
            var_file = acquired
        if args.development_observability:
            generate(args.dest, var_file, args.release, True)
        else:
            generate(args.dest, var_file, args.release)
    finally:
        if acquired is not None:
            acquired.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
