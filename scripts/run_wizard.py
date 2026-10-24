#!/usr/bin/env python3
"""Run the generation wizard and write the answers object.

Adapted from the reference wizard runner at commit 9ef4d03. Paths point at
``wizard/wizard.json`` and ``wizard/answers.schema.json``. The wizard UI does
not check the project-name pattern; finished answers are validated against
the schema here, and ``scripts/new_project.py`` validates them again.

Wyvern's finished-wizard stdout is a ``WizardResult`` object,
``{ "button", "data", "stack" }``, with no type wrapper. Every page returns
the full answers object, so ``data`` on a finish is that object.

Cancellation (``button`` ``cancel`` or ``dismissed``) writes no answers file
and returns 1. The last stderr line is ``GEN.WIZARD_CANCELLED``. The driver
then does not create a destination.

Acquisition failures end on the driver inventory. ``[run-wizard]`` diagnostics
may precede that line. The last line is the code an agent selects.
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
from typing import Any


def _force_utf8_console() -> None:
    """Make stdout/stderr UTF-8 with replacement before anything prints."""
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is None:
            continue
        try:
            reconfigure(encoding="utf-8", errors="replace")
        except (ValueError, OSError):
            pass


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent
WIZARD_JSON = REPO_ROOT / "wizard" / "wizard.json"
SCHEMA_PATH = REPO_ROOT / "wizard" / "answers.schema.json"
DASH = "\u2014"

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
        "WYVERN_BIN unset and wyvern not on PATH",
        "set WYVERN_BIN or install wyvern on PATH; cargo generate does not run",
    ),
    "GEN.WIZARD_CANCELLED": (
        "wizard cancelled or dismissed",
        "finish the wizard; the destination is not created",
    ),
}


class WizardError(RuntimeError):
    """A wizard acquisition failure with an inventory code."""

    def __init__(
        self,
        message: str,
        *,
        code: str = "GEN.SCHEMA_INVALID",
        cause: str | None = None,
        recovery: str | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        self.cause = cause
        self.recovery = recovery


def log(msg: str) -> None:
    print(f"[run-wizard] {msg}", file=sys.stderr)


def inventory_line(code: str, cause: str | None = None, recovery: str | None = None) -> str:
    """Return one ``<code> <cause> — <recovery>`` line."""
    default_cause, default_recovery = INVENTORY[code]
    return f"{code} {cause or default_cause} {DASH} {recovery or default_recovery}\n"


def emit_wizard_failure(exc: WizardError) -> int:
    """Write diagnostics, then the inventory line, and return the exit status."""
    default_cause, _recovery = INVENTORY[exc.code]
    shown = exc.cause or default_cause
    detail = str(exc).strip()
    if detail and detail != shown:
        log(detail)
    sys.stderr.write(inventory_line(exc.code, exc.cause, exc.recovery))
    return 1


def find_wyvern_bin() -> str:
    """Resolve Wyvern from ``WYVERN_BIN`` when set, otherwise from ``PATH``.

    An override that does not resolve is an error. There is no install prefix.
    """
    override = os.environ.get("WYVERN_BIN")
    if override:
        candidate = shutil.which(override) or (
            override if os.path.isfile(override) and os.access(override, os.X_OK) else None
        )
        if candidate:
            return candidate
        raise WizardError(
            f"WYVERN_BIN does not resolve to an executable: {override!r}",
            code="GEN.TOOL_MISSING",
            cause="WYVERN_BIN is set but does not resolve to an executable",
            recovery="set WYVERN_BIN to the wyvern binary path; cargo generate does not run",
        )
    candidate = shutil.which("wyvern")
    if candidate:
        return candidate
    raise WizardError(
        "wyvern binary not found (set $WYVERN_BIN to its path or put wyvern on PATH)",
        code="GEN.TOOL_MISSING",
        cause="WYVERN_BIN unset and wyvern not on PATH",
        recovery="set WYVERN_BIN or install wyvern on PATH; cargo generate does not run",
    )


def build_merged_descriptor(
    wizard_json_path: Path, prefill_path: Path | None, schema: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Load the descriptor and merge a partial prefill into ``config.prefill``.

    Keys absent from the schema ``properties`` are dropped with a stderr
    warning. The on-disk descriptor is not modified.
    """
    descriptor: dict[str, Any] = json.loads(wizard_json_path.read_text(encoding="utf-8"))
    if prefill_path is None:
        return descriptor, {}

    if not prefill_path.is_file():
        raise WizardError(f"--prefill not found: {prefill_path}")
    try:
        raw = json.loads(prefill_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise WizardError(f"failed to parse --prefill as JSON: {exc}") from exc
    if not isinstance(raw, dict):
        raise WizardError("--prefill file must contain a JSON object")

    properties: dict[str, Any] = schema.get("properties", {})
    cleaned: dict[str, Any] = {}
    for key, value in raw.items():
        if key not in properties:
            log(f"--prefill: dropping unknown key '{key}' (not in answers.schema.json)")
            continue
        cleaned[key] = value

    descriptor = dict(descriptor)
    config = dict(descriptor.get("config", {}))
    config["prefill"] = cleaned
    descriptor["config"] = config
    return descriptor, cleaned


def run_wyvern(wizard_json: Path, ui_root: Path, cwd: Path) -> str:
    """Run Wyvern and return its stdout text.

    ``ui_root`` is the wizard directory. Page paths in the descriptor resolve
    against it, including when ``wizard_json`` is a temporary merged copy.
    """
    wyvern_bin = find_wyvern_bin()
    with tempfile.TemporaryDirectory(prefix="run-wizard-") as tmp:
        stdout_path = Path(tmp) / "stdout.json"
        with open(stdout_path, "wb") as stdout_file:
            proc = subprocess.run(
                [wyvern_bin, str(wizard_json), "--ui-root", str(ui_root)],
                cwd=str(cwd),
                stdout=stdout_file,
                stderr=subprocess.PIPE,
                check=False,
            )
        stdout_text = stdout_path.read_text(encoding="utf-8")
        if proc.returncode != 0:
            stderr_text = proc.stderr.decode("utf-8", errors="replace") if proc.stderr else ""
            detail = stderr_text.strip() or "(no stderr)"
            raise WizardError(
                f"wyvern exited with status {proc.returncode}: {detail}",
                code="GEN.TOOL_MISSING",
                cause=f"wyvern exited with status {proc.returncode}",
                recovery="fix Wyvern or the wizard UI; cargo generate does not run",
            )
        return stdout_text


def parse_wizard_result(stdout_text: str) -> dict[str, Any]:
    stdout_text = stdout_text.strip()
    if not stdout_text:
        raise WizardError("wyvern produced no stdout output")
    try:
        result = json.loads(stdout_text)
    except json.JSONDecodeError as exc:
        raise WizardError(f"failed to parse wyvern stdout as JSON: {exc}") from exc
    if not isinstance(result, dict) or "button" not in result:
        raise WizardError(
            f"wyvern stdout does not look like a WizardResult (missing 'button'): {stdout_text[:200]!r}"
        )
    return result


def normalize_answers(result: dict[str, Any]) -> dict[str, Any]:
    """Return the finished answers object, or raise on cancellation."""
    button = result.get("button")
    if button in {"cancel", "dismissed"}:
        raise WizardError(
            "wizard cancelled or dismissed",
            code="GEN.WIZARD_CANCELLED",
        )
    if button != "finish":
        raise WizardError(
            f"wizard was not completed (button={button!r}); re-run and click Finish",
            code="GEN.WIZARD_CANCELLED",
        )

    data = result.get("data")
    if isinstance(data, dict) and data:
        return data

    stack = result.get("stack", [])
    if isinstance(stack, list):
        for entry in reversed(stack):
            if isinstance(entry, dict) and isinstance(entry.get("data"), dict) and entry["data"]:
                return entry["data"]

    raise WizardError("wizard result has no usable terminal page data")


def load_schema() -> dict[str, Any]:
    if not SCHEMA_PATH.is_file():
        raise WizardError(f"schema not found: {SCHEMA_PATH}")
    loaded = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    if not isinstance(loaded, dict):
        raise WizardError("answers schema must be a JSON object")
    return loaded


def project_name_rejected(answers: dict[str, Any], schema: dict[str, Any]) -> bool:
    """Return whether ``project_name`` fails the schema pattern.

    The pattern stays in ``answers.schema.json``. This function does not copy it.
    """
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        return False
    field = properties.get("project_name")
    if not isinstance(field, dict):
        return False
    pattern = field.get("pattern")
    name = answers.get("project_name")
    if not isinstance(pattern, str) or not isinstance(name, str):
        return False
    return re.fullmatch(pattern, name) is None


def validate_answers(answers: dict[str, Any], schema: dict[str, Any]) -> None:
    """Reject answers that fail the schema. The driver repeats this check.

    ``jsonschema`` is the validator. The driver already requires that package,
    so this runner does not keep a second checker.
    """
    if project_name_rejected(answers, schema):
        raise WizardError(INVENTORY["APP.INVALID_NAME"][0], code="APP.INVALID_NAME")
    try:
        import jsonschema
    except ImportError as exc:
        raise WizardError(
            "jsonschema is not installed",
            code="GEN.TOOL_MISSING",
            cause="jsonschema is not installed",
            recovery=(
                "install jsonschema per requirements-generation.txt; "
                "cargo generate does not run"
            ),
        ) from exc

    validator_cls = jsonschema.validators.validator_for(schema)
    validator_cls.check_schema(schema)
    validator = validator_cls(schema)
    errors = sorted(validator.iter_errors(answers), key=lambda err: list(err.path))
    if errors:
        messages = [
            f"{'/'.join(str(part) for part in err.path) or '<root>'}: {err.message}" for err in errors
        ]
        raise WizardError(
            "answers failed schema validation:\n  " + "\n  ".join(messages),
            code="GEN.SCHEMA_INVALID",
        )


def apply_defaults(answers: dict[str, Any], schema: dict[str, Any]) -> dict[str, Any]:
    """Fill schema defaults for keys the finished wizard left unset."""
    properties: dict[str, Any] = schema.get("properties", {})
    filled = dict(answers)
    for name, prop_schema in properties.items():
        if name not in filled and isinstance(prop_schema, dict) and "default" in prop_schema:
            filled[name] = prop_schema["default"]
    return filled


def summarize(answers: dict[str, Any]) -> str:
    return "\n".join(
        [
            f"project_name: {answers.get('project_name')}",
            f"db: {answers.get('db')}",
            f"mcp: {answers.get('mcp')}",
            f"licence: {answers.get('licence')}",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="where to write the answers JSON (stdout when omitted)",
    )
    parser.add_argument(
        "--print-only",
        action="store_true",
        help="validate and print the answers JSON; do not write --output",
    )
    parser.add_argument(
        "--wizard-json",
        type=Path,
        default=WIZARD_JSON,
        help=f"path to wizard.json (default: {WIZARD_JSON})",
    )
    parser.add_argument(
        "--prefill",
        type=Path,
        default=None,
        help="partial answers JSON merged into config.prefill for review",
    )
    args = parser.parse_args(argv)

    if not args.wizard_json.is_file():
        return emit_wizard_failure(WizardError(f"wizard.json not found: {args.wizard_json}"))

    merged_tmp_path: Path | None = None
    try:
        schema = load_schema()
        descriptor, cleaned_prefill = build_merged_descriptor(
            args.wizard_json, args.prefill, schema
        )
        wizard_json_to_run = args.wizard_json
        if args.prefill is not None:
            handle = tempfile.NamedTemporaryFile(
                mode="w",
                prefix="wizard-merged-",
                suffix=".json",
                delete=False,
                encoding="utf-8",
            )
            with handle:
                json.dump(descriptor, handle, indent=2)
                handle.write("\n")
            merged_tmp_path = Path(handle.name)
            wizard_json_to_run = merged_tmp_path
            log(f"--prefill applied: {len(cleaned_prefill)} key(s) merged into config.prefill")

        stdout_text = run_wyvern(wizard_json_to_run, args.wizard_json.parent, REPO_ROOT)
        result = parse_wizard_result(stdout_text)
        answers = apply_defaults(normalize_answers(result), schema)
        validate_answers(answers, schema)
    except WizardError as exc:
        return emit_wizard_failure(exc)
    finally:
        if merged_tmp_path is not None:
            merged_tmp_path.unlink(missing_ok=True)

    rendered = json.dumps(answers, indent=2, sort_keys=True) + "\n"
    if args.print_only or args.output is None:
        sys.stdout.write(rendered)
        log("answers printed; no output file written")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
        log(f"Wrote answers to {args.output}")
    log("Wizard complete:\n" + summarize(answers))
    return 0


if __name__ == "__main__":
    _force_utf8_console()
    raise SystemExit(main())
