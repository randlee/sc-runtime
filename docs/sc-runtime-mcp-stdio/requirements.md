# `sc-runtime-mcp-stdio` Requirements

These requirements define the `sc-runtime-mcp-stdio` crate surface and
constraints.

## Purpose

`sc-runtime-mcp-stdio` is the stdio JSON-RPC 2.0 MCP transport. It exists to
provide a zero-daemon MCP server path for sc-runtime tools. Its requirements
are deliberately narrow: it is a transport adapter, not an operation layer.

## Authoritative Requirement Sources

- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md) —
  Sections 6.3, 9, 11 (FR-02, FR-03), 12 (NF-02, NF-05)
- [architecture.md](./architecture.md)
- sc-ai-cli skill reference at `synaptic-canvas/packages/sc-ai-cli/skills/creating-ai-clis`

## Scope Rules

The following belong in `sc-runtime-mcp-stdio`:

- JSON-RPC 2.0 stdin reader and stdout writer
- MCP request framing and response framing
- Dispatch from MCP method name to the `sc-runtime-cli` operation layer
- Process lifetime management (stdin-closed detection, clean exit)

The following do not belong in `sc-runtime-mcp-stdio`:

- Operation logic (belongs in `sc-runtime-cli`)
- Daemon lifecycle, PID files, signal handling (belong in `sc-runtime-daemon`)
- HTTP or SSE transport (belongs in `sc-runtime-web`)
- Named pipe or Unix socket server (belongs in `sc-runtime-daemon`)
- Any dependency on `sc-runtime-daemon`

## Functional Requirements

### FR-STDIO-01 — Must Implement JSON-RPC 2.0 Over stdin/stdout

The crate must implement the JSON-RPC 2.0 protocol over stdin/stdout:

- Read newline-delimited JSON objects from stdin
- Parse each object as a JSON-RPC 2.0 request
- Dispatch to the operation layer
- Write a conforming JSON-RPC 2.0 response object to stdout for every request
- Handle malformed input by returning a JSON-RPC 2.0 parse error response
  (not by crashing)

**Rationale:** FR-03. The stdio MCP transport is the standard MCP deployment
path for tools without a daemon. Correct JSON-RPC 2.0 framing is required for
interoperability with any MCP host.

### FR-STDIO-02 — Must Not Depend on `sc-runtime-daemon`

`sc-runtime-mcp-stdio` must have zero compile-time dependency on
`sc-runtime-daemon`, directly or transitively. This is a hard boundary
enforced by `sc-lint-boundary` via the definitions in
`boundaries/sc-runtime-mcp-stdio/`. A pull request that introduces this
dependency must fail the CI lint gate.

**Rationale:** NF-02, ADR-05. A CLI + stdio MCP tool must not carry daemon
infrastructure in its binary. The boundary ensures that choosing
`mcp_stdio()` on a non-daemon builder configuration results in no daemon code
being compiled in.

### FR-STDIO-03 — Must Dispatch to the Same Operation Layer as the CLI

`sc-runtime-mcp-stdio` must dispatch MCP requests to the same operation
functions called by the CLI command handler. No MCP-specific operation logic
may exist. The operation layer is owned by `sc-runtime-cli`.

**Rationale:** FR-02. A single operation layer eliminates the possibility of
CLI and MCP diverging in behavior for the same operation.

### FR-STDIO-04 — Same JSON Fixtures Must Produce Identical Responses via CLI and via stdio MCP

The contract test suite must include fixtures exercised against both the CLI
`--json` path and the stdio MCP path. For a given operation (e.g.,
`runtime.status`), the `CommandEnvelope<T>` payload in the CLI `--json`
response and the `result` field of the MCP JSON-RPC response must be
identical. Divergence is a test failure.

**Rationale:** FR-02, ADR-06. Parity is the enforcement mechanism for the
no-reshaping rule at the stdio MCP transport boundary.

### FR-STDIO-05 — Process Lifetime Must Equal MCP Client Connection Lifetime

The `sc-runtime-mcp-stdio` server must exit when stdin is closed. It must not
attempt to maintain state across MCP host reconnects. The process must exit
cleanly (exit code 0) on normal stdin-close. It must not leave orphan
background tasks running after stdin closes.

**Rationale:** FR-03. The MCP host expects the server process to exit when the
connection ends. Persistent background activity after disconnect would
constitute a resource leak and violate the expected process model.

## Non-Functional Requirements

### NF-STDIO-01 — `unsafe_code = "forbid"`

The `sc-runtime-mcp-stdio` crate manifest must set `#![forbid(unsafe_code)]`.

### NF-STDIO-02 — No Daemon Layer in the Dependency Tree

The `sc-runtime-daemon` crate must not appear anywhere in the transitive
dependency graph of `sc-runtime-mcp-stdio`. This is verified by the
`sc-lint-boundary` gate.

## Boundary Governing Document

The boundary rules for `sc-runtime-mcp-stdio` are defined in:

```
boundaries/sc-runtime-mcp-stdio/
```

The critical rule in that definition: `sc-runtime-daemon` is a forbidden
dependency. The sc-lint-boundary CI gate reads this definition and fails on
any violation.

## Related Docs

- [architecture.md](./architecture.md)
- [../sc-runtime-cli/requirements.md](../sc-runtime-cli/requirements.md)
- [../sc-runtime-core/requirements.md](../sc-runtime-core/requirements.md)
- [../../boundaries/sc-runtime-mcp-stdio/](../../boundaries/sc-runtime-mcp-stdio/)
- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md)
