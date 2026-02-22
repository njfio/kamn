# Spec: #5692 Implement MCP JSON-RPC Stdio Protocol in `kamn-mcp-server`

- Issue: #5692
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`crates/kamn-mcp-server` currently executes line-delimited JSON requests, but it
does not yet expose the MCP JSON-RPC stdio surface required for direct
integration with MCP-compatible agents. Without `initialize`, `tools/list`, and
`tools/call` JSON-RPC handling plus `Content-Length` framing, the server cannot
act as a portable MCP tool server per the PRD.

## Scope
### In Scope
- Add stdio MCP framing support (`Content-Length` header + JSON body parsing).
- Implement JSON-RPC methods:
  - `initialize`
  - `tools/list`
  - `tools/call`
- Route `tools/call` to existing backend tool dispatch.
- Return deterministic JSON-RPC error envelopes for unsupported methods and
  malformed payloads.
- Preserve existing newline JSON dispatch mode for existing contract tests.

### Out of Scope
- Transport modes beyond stdio (SSE/WebSocket).
- Authentication redesign or key-loading semantics.
- New tool inventory; must use existing 12-tool registry.
- Dependency additions.

## Acceptance Criteria
### AC-1 MCP initialize
Given a valid MCP framed JSON-RPC `initialize` request,
When `kamn-mcp-server` processes it,
Then it returns a framed JSON-RPC success response with server/tool capability
metadata.

### AC-2 MCP tools/list
Given a valid MCP framed JSON-RPC `tools/list` request,
When the request is processed,
Then the response includes the full 12-tool registry and deterministic descriptor
fields for each tool.

### AC-3 MCP tools/call dispatch
Given a valid MCP framed JSON-RPC `tools/call` request for a supported tool,
When processed,
Then the request is routed through existing backend dispatch and returns a
framed JSON-RPC success result reflecting backend output.

### AC-4 JSON-RPC error handling
Given unsupported methods or malformed method payloads,
When processed,
Then the server emits framed JSON-RPC error responses with deterministic
`code`/`message` markers.

### AC-5 Legacy compatibility
Given existing non-framed newline JSON tool requests,
When processed,
Then current line-mode dispatch remains functional and deterministic.

## Conformance Cases
- C-01 (AC-1): Framed `initialize` request yields framed JSON-RPC success with
  `serverInfo` and tool capability markers.
- C-02 (AC-2): Framed `tools/list` request yields framed response listing all
  names from `MCP_TOOL_NAMES`.
- C-03 (AC-3): Framed `tools/call` (`health`) yields framed JSON-RPC success and
  includes tool result payload.
- C-04 (AC-4): Unsupported method yields JSON-RPC `method_not_found` error
  envelope.
- C-05 (AC-4): Malformed `tools/call` payload yields JSON-RPC `invalid_params`
  error envelope.
- C-06 (AC-5): Existing line-mode request still dispatches and returns existing
  non-framed result.

## Success Metrics / Observable Signals
- `cargo test -p kamn-mcp-server` passes with new protocol conformance tests.
- `kamn-mcp-server` emits framed responses for framed MCP input.
- Existing dispatch and inventory tests remain green without behavior regressions.
