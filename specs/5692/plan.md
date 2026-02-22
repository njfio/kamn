# Plan: #5692 MCP JSON-RPC Stdio Protocol in `kamn-mcp-server`

## Approach
1. Add RED conformance tests for framed MCP JSON-RPC handling:
   - `initialize`
   - `tools/list`
   - `tools/call`
   - error paths
   - legacy line-mode compatibility
2. Introduce protocol helpers in `kamn-mcp-server` to:
   - split framed input stream into JSON payloads
   - process JSON-RPC request envelopes
   - encode framed JSON-RPC responses
3. Reuse existing `build_tool_registry` and `dispatch_tool_request_json`.
4. Update binary execution path to auto-detect framed input and emit framed
   output while preserving existing newline mode behavior.
5. Run targeted and crate-wide tests, then formatting/lint checks.

## Affected Modules
- `crates/kamn-mcp-server/src/main.rs`
- `crates/kamn-mcp-server/src/lib.rs`
- `crates/kamn-mcp-server/src/dispatch.rs` (as needed for reusable helpers)
- `crates/kamn-mcp-server/src/tools.rs` (read-only registry reuse)
- `crates/kamn-mcp-server/tests/*` (new/updated conformance tests)

## Risks and Mitigations
- Risk: brittle ad-hoc JSON extraction for JSON-RPC envelopes.
  Mitigation: keep parsing constrained to deterministic test contracts and add
  malformed input tests for expected failure mode.

- Risk: response framing errors (bad byte lengths) breaking MCP clients.
  Mitigation: conformance tests assert `Content-Length` correctness and framed
  body extraction.

- Risk: regressions in existing line-mode flow.
  Mitigation: keep line-mode code path and explicit compatibility conformance
  coverage.

## Interfaces / Contracts
- New internal protocol contract:
  - framed stream in (`Content-Length` + JSON body)
  - framed JSON-RPC response out
- Existing line-mode contract remains unchanged.
- Existing tool registry and backend dispatch contracts remain unchanged.

## ADR
- Not required: no new dependency, wire-format scope limited to MCP stdio
  framing and JSON-RPC method routing within existing crate boundaries.
