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

SCHEMA_NAME = "wizard/answers.schema.json"
PATCH_CRATES = ("sc-config", "sc-transport", "sc-command", "sc-runtime")
PIPELINE = (
    "values",
    "cargo-generate",
    "sc-compose",
    "setup",
    "lint",
    "test",
    "build-cli",
    "build-daemon",
)
COMPOSE_DOCS = ("AGENTS.md.j2", "CLAUDE.md.j2")

INVENTORY = {
    "GEN.SCHEMA_INVALID": (
        "answers fail wizard/answers.schema.json",
        "fix the file; cargo is not called; the destination is not created",
    ),
    "GEN.DB_RESERVED": (
        "db is postgres or both",
        "set db to sqlite; the destination is not created",
    ),
    "APP.INVALID_NAME": (
        "name fails the project-name pattern in wizard/answers.schema.json",
        "fix the name to that pattern; the destination is not created",
    ),
    "GEN.TOOL_MISSING": (
        "SC_COMPOSE unset and sc-compose not on PATH",
        "set SC_COMPOSE or install sc-compose on PATH; cargo generate does not run",
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
    fail(
        "GEN.STEP_FAILED",
        f"step is {step}",
        "stderr names that step; fix it; later steps do not run",
    )


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


def reserved_database(schema: dict, answers: dict) -> bool:
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        return False
    for spec in properties.values():
        if not isinstance(spec, dict):
            continue
        reserved = spec.get("x-v01-reserved")
        if not isinstance(reserved, list):
            continue
        role = spec.get("x-role")
        key = next(
            (name for name, value in properties.items() if value is spec),
            None,
        )
        if role != "database" or key is None:
            continue
        return answers.get(key) in reserved
    return False


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


def json_type_ok(expected: object, value: object) -> bool:
    if expected == "string":
        return isinstance(value, str)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "array":
        return isinstance(value, list)
    if expected == "object":
        return isinstance(value, dict)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    return False


def schema_accepts(schema: dict, answers: dict) -> bool:
    if schema.get("type") not in (None, "object"):
        return False
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        return False
    required = schema.get("required", [])
    if not isinstance(required, list) or any(key not in answers for key in required):
        return False
    if schema.get("additionalProperties") is False and any(
        key not in properties for key in answers
    ):
        return False
    for key, value in answers.items():
        spec = properties.get(key)
        if not isinstance(spec, dict):
            return False
        expected = spec.get("type")
        if isinstance(expected, str) and not json_type_ok(expected, value):
            return False
        enum = spec.get("enum")
        if isinstance(enum, list) and value not in enum:
            return False
        pattern = spec.get("pattern")
        if isinstance(pattern, str) and isinstance(value, str):
            if re.compile(pattern).fullmatch(value) is None:
                return False
        minimum = spec.get("minLength")
        if isinstance(minimum, int) and isinstance(value, str) and len(value) < minimum:
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
    if reserved_database(schema, answers):
        fail("GEN.DB_RESERVED")
    return apply_defaults(schema, answers)


def locate_sc_compose() -> str | None:
    override = os.environ.get("SC_COMPOSE")
    if override:
        return override
    return shutil.which("sc-compose")


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


def run_command(argv: list[str], cwd: Path) -> int:
    completed = subprocess.run(
        argv,
        cwd=cwd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return completed.returncode


def cargo_generate(root: Path, dest: Path, name: str, values: Path) -> None:
    try:
        dest.mkdir()
    except OSError:
        fail_step("cargo-generate")
    code = run_command(
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
            "--force",
            "--no-workspace",
        ],
        root,
    )
    if code != 0:
        fail_step("cargo-generate")
    # Liquid tag lines leave a blank line. Format before the recipes so
    # `just lint` sees a rustfmt-clean tree for both mcp values.
    if run_command(["cargo", "fmt", "--all"], dest) != 0:
        fail_step("cargo-generate")


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
    for name in PATCH_CRATES:
        crate = (root / "crates" / name).resolve()
        if not (crate / "Cargo.toml").is_file():
            fail_step("cargo-generate")
        lines.append(f'{name} = {{ path = "{crate.as_posix()}" }}')
    try:
        manifest.write_text(text.rstrip() + "\n" + "\n".join(lines) + "\n", encoding="utf-8")
    except OSError:
        fail_step("cargo-generate")


def render_documents(dest: Path, compose: str, answers_path: Path) -> None:
    for name in COMPOSE_DOCS:
        source = dest / name
        output = dest / name.removesuffix(".j2")
        code = run_command(
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
        )
        if code != 0:
            fail_step("sc-compose")
        try:
            source.unlink()
        except OSError:
            fail_step("sc-compose")


def run_recipes(dest: Path) -> None:
    for step in ("setup", "lint", "test", "build-cli", "build-daemon"):
        if run_command(["just", step], dest) != 0:
            fail_step(step)


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


def generate(dest: Path, var_file: Path, release: bool) -> None:
    root = repo_root()
    schema = load_schema(root)
    answers = validate(schema, load_json(var_file))
    if dest.exists():
        fail("GEN.DEST_CONFLICT")
    compose = locate_sc_compose()
    if compose is None:
        fail("GEN.TOOL_MISSING")
    name = answers[PROJECT_NAME_KEY]
    if not isinstance(name, str):
        fail("GEN.SCHEMA_INVALID")
    values = write_values(answers)
    composed = write_answers_for_compose(answers)
    try:
        cargo_generate(root, dest, name, values)
        if not release_mode(release):
            apply_checkout_patch(dest, root)
        render_documents(dest, compose, composed)
        run_recipes(dest)
    finally:
        values.unlink(missing_ok=True)
        composed.unlink(missing_ok=True)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("dest", nargs="?", type=Path)
    parser.add_argument("--var-file", type=Path)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--patch-checkout", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    if args.patch_checkout is not None:
        apply_checkout_patch(args.patch_checkout, repo_root())
        return
    if args.dest is None or args.var_file is None:
        fail("GEN.SCHEMA_INVALID")
    generate(args.dest, args.var_file, args.release)


if __name__ == "__main__":
    main()
