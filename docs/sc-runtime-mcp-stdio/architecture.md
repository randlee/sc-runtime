# `sc-runtime-mcp-stdio` Architecture

This document records the architecture of the `sc-runtime-mcp-stdio` crate.

## Role

`sc-runtime-mcp-stdio` is the stdio MCP transport for sc-runtime. It
implements JSON-RPC 2.0 over stdin/stdout. It runs in the foreground; there is
no background process, no PID file, no daemon socket, and no persistent
connection management.

The crate is the primary integration point for MCP hosts (Claude Desktop, MCP
proxy, any JSON-RPC 2.0 client). It starts when the process starts, serves
until the MCP client disconnects (stdin closes), and exits cleanly.

## What stdio MCP Is

The Model Context Protocol (MCP) defines a JSON-RPC 2.0 transport over
stdin/stdout for tool servers that run as child processes of an MCP host.

The host (e.g., Claude Desktop) spawns the tool process. The tool process:

1. reads newline-delimited JSON-RPC 2.0 requests from stdin
2. dispatches each request to the operation layer
3. writes newline-delimited JSON-RPC 2.0 responses to stdout
4. exits when stdin is closed (MCP client disconnected)

The stdio reader processes one complete newline-delimited JSON line at a time.
Maximum line length: **1 MiB** (1,048,576 bytes). Lines exceeding this limit
return a JSON-RPC `-32700 Parse error` response and close the connection. This
limit prevents unbounded memory allocation from a malformed or malicious client.

There are no sockets, no listening ports, and no background processes. The
process lifetime equals the MCP client connection lifetime.

## No Daemon Dependency

`sc-runtime-mcp-stdio` must not depend on `sc-runtime-daemon`. This is a hard
compile-time boundary enforced by `sc-lint-boundary`. The boundary definition
is in `boundaries/sc-runtime-mcp-stdio/` (see
[../../boundaries/sc-runtime-mcp-stdio/](../../boundaries/sc-runtime-mcp-stdio/)).

### Why This Boundary Exists

A tool that combines CLI and stdio MCP without a daemon should have no
background process footprint. If `sc-runtime-mcp-stdio` pulled in
`sc-runtime-daemon`, any consumer of `sc-runtime-mcp-stdio` would transitively
depend on daemon lifecycle code, PID management, signal handling, and the IPC
server — none of which are relevant to a stdio-only tool.

ADR-05 records this decision: "A CLI + stdio MCP tool has no background
process, no PID file, no socket — simpler deployment and no lifecycle
management overhead."

The boundary means a consumer can choose the composability configuration
`CLI + stdio MCP` and get exactly those two capabilities with no daemon
overhead in the binary.

## Operation Dispatch

`sc-runtime-mcp-stdio` dispatches to the same operation layer as the CLI.
There is no MCP-specific operation logic. The dispatch path is:

```
stdin → JSON-RPC 2.0 deserialize → CommandEnvelope<T> → operation layer
operation layer → CommandEnvelope<T> → JSON-RPC 2.0 serialize → stdout
```

The operation layer is owned by `sc-runtime-cli`. `sc-runtime-mcp-stdio`
is a transport adapter over that layer.

### Zero Reshaping

The same `CommandEnvelope<T>` and `CliError` types used by the CLI `--json`
path are used for MCP responses. No transport-specific DTOs exist.
No conversion or mapping occurs between the operation result and the MCP
response payload.

This is the enforcement point for FR-02 (no reshaping at transport boundaries)
in the stdio MCP transport.

### JSON-RPC 2.0 Framing

The JSON-RPC 2.0 envelope is added by `sc-runtime-mcp-stdio` as a thin
framing layer around `CommandEnvelope<T>`:

```json
// Request from MCP host (stdin)
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "tools/call",
  "params": {
    "name": "runtime.status",
    "arguments": {}
  }
}

// Response to MCP host (stdout)
{
  "jsonrpc": "2.0",
  "id": "1",
  "result": {
    "ok": true,
    "command": "runtime.status",
    "data": { "state": "running", "pid": 12345, "uptime_secs": 300 },
    "error": null,
    "diagnostics": []
  }
}
```

The `result` field carries the `CommandEnvelope<T>` directly. This preserves
the machine-readable contract for any client that reads both the CLI `--json`
output and the MCP response.

## Process Lifetime

The process starts when the MCP host spawns it. It exits when stdin is closed
— which occurs when the MCP host disconnects or terminates. There is no
explicit shutdown command or keepalive mechanism.

```
MCP host spawns process
  → process reads from stdin
  → process writes to stdout
  → stdin closes (MCP host disconnect)
  → process exits 0
```

The `CancellationToken` from `sc-runtime-core` is wired to the stdin-closed
event so that any in-flight operation is given a chance to complete cleanly
before the process exits.

## MCP Contract Parity

Per the sc-ai-cli skill: "test the same JSON fixtures against the CLI path and
the MCP path with no contract reshaping between them."

Contract parity means:

- the same JSON request payload that produces a `CommandEnvelope<StatusResult>`
  via `my-tool status --json` must produce an identical `CommandEnvelope<StatusResult>`
  when submitted via stdio MCP
- the same error scenarios that produce a specific `CliError.code` via CLI
  must produce the same `CliError.code` in the MCP response
- the fixture set used in CLI integration tests is also run against the stdio
  MCP path in the same test suite

Divergence between CLI and MCP responses for the same operation is a bug.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `sc-runtime-core` | `PluginContext`, `ScRuntimeError`, `CancellationToken` wiring |
| `sc-runtime-cli` | Operation layer, `CommandEnvelope<T>`, `CliError`, `CommandId` |
| `serde_json` | JSON-RPC 2.0 framing serialization |
| `tokio` | Async stdin/stdout I/O |

`sc-runtime-daemon` is explicitly excluded. No other daemon-layer crate may
appear in this crate's dependency tree.

## Related Docs

- [requirements.md](./requirements.md)
- [../sc-runtime-cli/architecture.md](../sc-runtime-cli/architecture.md)
- [../sc-runtime-core/architecture.md](../sc-runtime-core/architecture.md)
- [../../boundaries/sc-runtime-mcp-stdio/](../../boundaries/sc-runtime-mcp-stdio/)
- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md) — Sections 6.3, 9, 11 (FR-03)
