# Plan: Issue #6004

## Approach
1. Introduce typed JSON-RPC request decoding (`serde_json::Value` + focused helper structs/enums) in `protocol.rs`.
2. Refactor `process_jsonrpc_request` and `process_tools_call` to operate on parsed request objects instead of raw request JSON strings.
3. Replace ad hoc field scans in dispatch-payload builder with typed parameter map extraction.
4. Preserve framed decoding/encoding behavior and error code mapping.
5. Add RED/GREEN tests for escaped + nested params and malformed JSON regressions.

## Affected Modules
- `crates/kamn-mcp-server/src/protocol.rs`

## Risks / Mitigations
- Risk: behavior drift in JSON-RPC envelope formatting.
  Mitigation: keep existing response constructors and add regression tests against framed output behavior.
- Risk: optional parameter extraction mismatch with historical behavior.
  Mitigation: maintain same supported key set and add direct tests for expected mapping.
- Risk: parse errors shifting from invalid-request to parse-error unexpectedly.
  Mitigation: assert explicit error-code outcomes for malformed JSON vs schema-invalid requests.

## Interfaces / Contracts
- MCP stdio framing unchanged.
- JSON-RPC method names and output shape unchanged.
- Parsing internals move to serde-backed contract.
