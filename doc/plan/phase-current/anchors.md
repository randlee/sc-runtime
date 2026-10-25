# Anchors - phase-current

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `doc/plan/phase-current/intent.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | locked | Bootstrap database commands the consumer can disable | 2026-10-25 |

## ANC-001

Anchor: The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface. An application command calls the same backend operation through its own CLI command, MCP tool, and HTTP route.

> the 'general database commands' would be expected to be disabled by the customer. they are simply a bootstrap tool and could easily be 'replaced' w/ a command that has better ux and does a similar call on the back-end. i.e. delete row <table> <id> would be a general command which could be used 'under the hood' w/ a different cli/mcp/http call.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Bootstrap database commands the consumer can disable

Scope: a bead that edits template CLI, MCP, or HTTP commands

Deterministic check to pair with the development gate: The generated template registers a general delete-row command on the CLI, on MCP, and over HTTP, and a generation flag omits those three registrations while the backend operation remains callable.

Beads: anchor `run-8bp`, plan questions `run-8bp.1`, dev questions `run-8bp.2`
