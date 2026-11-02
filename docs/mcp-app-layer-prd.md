# sc-runtime MCP App Layer — PRD

Oct 29, 2026 · @Rand Lee

## Summary

sc-runtime will scaffold a tokio web app whose MCP server also serves an MCP App: a single-file HTML View that renders inline in Claude Desktop chat, claude.ai and ChatGPT. Control traffic goes over MCP tools. Bulk and streaming data goes directly from the View to the app's HTTPS backend, which is declared in the View's CSP.

**Goals**

- One scaffold command produces a server that works as a plain MCP server and as an MCP App, with no host-specific code paths.
- The same View runs in Claude and ChatGPT using only the MCP Apps standard (`_meta.ui.*`, `ui/*` bridge).
- Local development works end to end before anything is deployed.

**Non-goals**

- Rendering in Claude Code (CLI or Desktop Code tab): Claude Code does not render MCP Apps, and the request for it was closed as not planned.
- ChatGPT-only features (`window.openai`), nested iframes, and a self-hosted MCP Apps host.

## Verified findings

Both questions resolve against a localhost-only design. Remote hosts need a public HTTPS MCP endpoint, and the View cannot reliably reach `http://localhost`.

**Q1. Does the host need a public HTTPS MCP endpoint?** Yes, for any connector added through settings.

| Host | How it connects | Localhost works? |
| --- | --- | --- |
| claude.ai and Claude Desktop custom connector | Anthropic's cloud opens the streamable-HTTP connection, not your machine ([custom connectors](https://support.claude.com/en/articles/11175166)) | No. Public HTTPS with a trusted certificate is required ([issue #9](https://github.com/anthropics/claude-ai-mcp/issues/9)) |
| Claude Desktop local config | `claude_desktop_config.json` speaks stdio only; a local HTTP server is reached through an `mcp-remote` stdio bridge ([guide](https://startdebugging.net/2026/05/fix-http-mcp-server-url-wont-connect-in-claude-desktop/)) | Yes, on that one machine, Desktop only |
| ChatGPT developer mode | Connects over the public internet to an HTTPS `/mcp` endpoint; tunnel localhost for development ([guide](https://www.getdrio.com/blog/how-to-add-mcp-tools-to-chatgpt)) | No |

**Q2. Can the View reach the app backend on localhost?** Not reliably. Use an HTTPS origin.

- The View's origin is `https://<32-hex>.claudemcpcontent.com`, even for a local stdio server, and `connect-src` contains every declared `connectDomains` entry, as measured in Claude Desktop ([steel-mcp-server notes](https://github.com/steel-dev/steel-mcp-server/blob/main/NOTES.md)).
- Claude's sandbox CSP includes `upgrade-insecure-requests` and `block-all-mixed-content`, and a declared `http://localhost` origin was dropped from the enforced policy ([claude-code #105199](https://github.com/anthropics/claude-code/issues/105199), [claude-ai-mcp #40](https://github.com/anthropics/claude-ai-mcp/issues/40)).
- Maintainers confirm `connectDomains` and `resourceDomains` work for HTTPS origins. `frameDomains` (nested iframes) is not allowed on claude.ai for security reasons (#40).
- A backend that allowlists origins must allow the View's stable origin, set with `_meta.ui.domain` ([CSP and CORS](https://apps.extensions.modelcontextprotocol.io/api/documents/csp-and-cors.html)). Claude now validates `ui.domain` against its hash-based `*.claudemcpcontent.com` form (#40).

## ADR-001: Split control and data planes behind one HTTPS origin

**Status:** Proposed

**Context.** An MCP App View is a sandboxed iframe on a host-owned origin. It reaches its server two ways: `tools/call` proxied by the host over postMessage, or direct fetch/WebSocket to origins listed in `connectDomains`. Host-proxied calls suit actions the model should see. They are a poor fit for image frames, telemetry streams or large payloads. Remote hosts reach the MCP server from their cloud, and the View can only reach HTTPS origins.

&#91;embedded content: sc-runtime MCP App architecture · control and data planes\]

The host proxies tool calls; the View's frames and streams bypass the host and hit the tokio app directly.

**Decision.**

1. **Control plane over MCP.** Every user action and state change is an MCP tool. Model-facing tools use `visibility: ["model","app"]`. UI-only handlers use `["app"]`.
2. **Data plane direct.** Bulk and streaming data moves over HTTPS or WSS from the View to the same tokio app. Its origin is the only entry in `connectDomains`.
3. **One process, one origin.** A single tokio binary serves `/mcp` (streamable HTTP), `/api/*` and `/ws/*` on one HTTPS origin. The View HTML is compiled into the binary and served as the `ui://` resource.
4. **Standard only.** Use `_meta.ui.*` and the `ui/*` bridge. OpenAI aliases (`openai/outputTemplate`) are off by default, behind an opt-in flag.
5. **Data-plane auth via tool result.** A tool result carries a short-lived, scoped token in `structuredContent`. The View presents it on every data-plane request. Cookies are not used, because the View's origin is host-owned.

**Alternatives considered.**

| Option | Why not chosen |
| --- | --- |
| All traffic through MCP tools | No CSP or CORS work, but every byte is proxied through the host, which may block or require approval for app messages. Wrong for streams and frames. |
| Separate API service on its own origin | A second origin to deploy, certify, allowlist and keep in sync, for no gain over path routing. |
| ChatGPT Apps SDK (`window.openai`) | Locks the View to one host. ChatGPT itself recommends the MCP Apps standard first. |

**Consequences.**

- Local-only development is limited to Claude Desktop through stdio or `mcp-remote`. claude.ai and ChatGPT need an HTTPS tunnel, so the scaffold must include a tunnel workflow.
- The backend must send CORS headers for the View's origin and validate the data-plane token. It cannot rely on origin checks alone, because the token is the real credential.
- The View degrades gracefully: on a host without UI support, the tools still return useful text.

## Critical requirements

Each requirement is a MUST, and each has a mechanical check the scaffold ships with.

| ID | Requirement | Mechanical check |
| --- | --- | --- |
| REQ-01 | The UI resource uses a `ui://` URI and mime type `text/html;profile=mcp-app`, and contains one valid HTML5 document. | Integration test runs `resources/read` and asserts the URI, mime type and HTML parse. |
| REQ-02 | Every UI-bearing tool sets `_meta.ui.resourceUri` pointing at a registered `ui://` resource. | Startup assertion: every referenced URI resolves, or the server refuses to start. |
| REQ-03 | Every tool sets `_meta.ui.visibility`, and handlers called only by the UI use `["app"]`. | Lint or test that fails when the field is missing. |
| REQ-04 | Every tool result includes meaningful text `content`, so a host without UI support still gets an answer. | Test calls each tool with a client that does not advertise UI and asserts the text is non-empty. |
| REQ-05 | The server reads the client's `extensions["io.modelcontextprotocol/ui"]` capability and only attaches UI metadata when it lists `text/html;profile=mcp-app`. | Two-client test: one with the capability, one without. |
| REQ-06 | `_meta.ui.csp` declares every origin the View contacts, all HTTPS or WSS, and none `http://`. | Config validation rejects non-HTTPS origins at build time. |
| REQ-07 | `_meta.ui.csp` and `_meta.ui.domain` are set on the `resources/read` contents (and mirrored on the resource listing). | Snapshot test of `resources/read` and `resources/list` output. |
| REQ-08 | The View bundle is self-contained: no static `<script src>`; external code loads dynamically from a declared `resourceDomains` origin. | Build step fails on any static `<script src>` in the output HTML. |
| REQ-09 | Data-plane endpoints reject requests without a valid, unexpired token issued in a tool result. | Test: unauthenticated `/api` and `/ws` requests return 401. |
| REQ-10 | CORS on `/api` and `/ws` allows only configured View origins, never `*` when credentials are used. | Header test against the allowed origin and a foreign origin. |
| REQ-11 | `/mcp`, `/api` and `/ws` are served by one tokio process on one HTTPS origin. | Deployment smoke test hits all three paths on one host. |
| REQ-12 | No host-specific code paths by default. OpenAI aliases are only emitted behind an explicit flag. | Grep check: no `openai/` keys in output unless the flag is set. |

## Important details

**Metadata the scaffold emits**

| Field | Where | Value or rule |
| --- | --- | --- |
| `uri` | Resource | `ui://<app>/view.html` |
| `mimeType` | Resource | `text/html;profile=mcp-app` |
| `_meta.ui.csp.connectDomains` | `resources/read` contents | The app's HTTPS and WSS origin; maps to `connect-src` |
| `_meta.ui.csp.resourceDomains` | `resources/read` contents | CDN origins for scripts, styles, images, fonts; maps to `img-src`, `script-src` and related |
| `_meta.ui.domain` | `resources/read` contents | Stable sandbox origin. For Claude: the first 32 hex characters of SHA-256 of the MCP server URL, plus `.claudemcpcontent.com` |
| `_meta.ui.prefersBorder` | `resources/read` contents | Optional |
| `_meta.ui.resourceUri` | Tool descriptor | The `ui://` URI. The flat `_meta["ui/resourceUri"]` is deprecated |
| `_meta.ui.visibility` | Tool descriptor | `["model","app"]` (default) or `["app"]` |

**How the View talks to the server**

- The View is an MCP client over postMessage. It can send `tools/call`, `resources/read`, `notifications/message` and `ping`, all proxied by the host.
- The host rejects app-initiated `tools/call` for tools without `"app"` in their visibility.
- The JS `@modelcontextprotocol/ext-apps` `App` class implements the bridge. The scaffold's frontend uses it as is; no Rust port is needed for the View.

**Default CSP when `ui.csp` is omitted:** `connect-src 'none'`, scripts and styles `'self' 'unsafe-inline'`, images and media `'self' data:`. Hosts may restrict further but must not allow undeclared domains.

**Host differences that matter**

| Host | Renders MCP Apps | Notes |
| --- | --- | --- |
| Claude Desktop chat, claude.ai | Yes | `frameDomains` not allowed on claude.ai; `ui.domain` validated against the hash form |
| ChatGPT | Yes | Same iframe-and-bridge standard; `window.openai` optional; `openai/outputTemplate` accepted as an alias |
| Claude Code (CLI, Desktop Code tab) | No | Text fallback only |

**Dev loop the scaffold provides**

1. `basic-host` from the ext-apps repo for host-free View testing.
2. Claude Desktop through `mcp-remote` against `http://localhost/mcp`. The data plane still needs an HTTPS origin (REQ-06).
3. An HTTPS tunnel profile for claude.ai and ChatGPT, with the tunnel origin injected into `connectDomains`.

## Open questions and risks

**Open questions**

- [ ] Does `rmcp` (or our chosen Rust MCP crate) let us set `_meta` on tool descriptors and on `resources/read` contents, and advertise or read the `extensions` capability? Not yet verified.
- [ ] What `ui.domain` format does ChatGPT accept? The ext-apps CSP guide defers to OpenAI's docs.
- [ ] What token lifetime and refresh path should the data plane use for long-running Views?
- [ ] Is a per-developer stable tunnel domain needed so the Claude `ui.domain` hash (derived from the MCP URL) stays constant across sessions?

**Risks**

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Host CSP handling changes without notice (#40 reports a `ui.domain` regression) | View loses data-plane access | Contract tests per host; a fallback to tool-only mode for small payloads |
| ChatGPT Plus and Pro developer mode allows read and fetch only for custom MCP; full write support centers on Business, Enterprise and Edu ([Boomi guide](https://help.boomi.com/docs/Atomsphere/Platform/Connect_chatgpt_setup)) | Write tools may not work for some ChatGPT users | Document plan limits; keep write tools optional |
| Public HTTPS endpoint exposes the MCP server | Unauthorized tool calls | OAuth or bearer auth on `/mcp`, separate from the data-plane token |
| Teams expect Claude Code to render the View | Confusion during development | State the non-goal in the scaffold README; the text fallback is the Claude Code experience |

## Sources

- [MCP Apps spec 2026-01-26 (ext-apps repo)](https://github.com/modelcontextprotocol/ext-apps/blob/main/specification/2026-01-26/apps.mdx)
- [MCP Apps CSP and CORS guide](https://apps.extensions.modelcontextprotocol.io/api/documents/csp-and-cors.html)
- [MCP Apps compatibility in ChatGPT (OpenAI)](https://developers.openai.com/apps-sdk/mcp-apps-in-chatgpt)
- [claude-ai-mcp #9: Desktop rejects http on localhost](https://github.com/anthropics/claude-ai-mcp/issues/9)
- [claude-ai-mcp #40: CSP fields and ui.domain on claude.ai](https://github.com/anthropics/claude-ai-mcp/issues/40)
- [claude-code #105199: Desktop Code tab CSP](https://github.com/anthropics/claude-code/issues/105199)
- [Claude Code request to render MCP Apps, closed not planned](https://claudeissues.com/issue/48132-feature-preview-tool-should-support-rendering-mcp-apps-ext-apps)
- [steel-mcp-server: measured Claude CSP](https://github.com/steel-dev/steel-mcp-server/blob/main/NOTES.md)
- [Fix: HTTP MCP URL in Claude Desktop (stdio vs HTTP)](https://startdebugging.net/2026/05/fix-http-mcp-server-url-wont-connect-in-claude-desktop/)
- [How to add MCP tools to ChatGPT](https://www.getdrio.com/blog/how-to-add-mcp-tools-to-chatgpt)
- [Boomi: connect ChatGPT to a custom MCP server](https://help.boomi.com/docs/Atomsphere/Platform/Connect_chatgpt_setup)
