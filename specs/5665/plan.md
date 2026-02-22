# Plan: #5665 Activate MCP Server Execution and List-Messages Parity

## Approach
1. Add RED tests in `kamn-sdk` for channel messages endpoint composition and decoding.
2. Add RED tests in `kamn-agent-lib` to verify `list_messages` behavior and regression from unsupported stub.
3. Implement SDK + agent-lib list-messages path.
4. Add RED tests in `kamn-mcp-server` for supported and unsupported tool dispatch payloads.
5. Implement MCP request model, dispatch layer, and structured response encoding.
6. Run targeted test gates first, then formatting/lint gates for touched crates.

## Affected Modules
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-mcp-server/src/lib.rs`
- `crates/kamn-mcp-server/src/main.rs`
- New/updated crate tests under `crates/kamn-sdk/tests/`, `crates/kamn-agent-lib/tests/`, `crates/kamn-mcp-server/tests/`

## Risks and Mitigations
- Risk: service route schema mismatch for list-messages decoding.
- Mitigation: response fixture coverage in SDK tests and agent-lib integration tests.
- Risk: MCP payload format drift from expected agent runtime usage.
- Mitigation: strict request/response structs with serialization tests and deterministic error envelopes.
- Risk: touching multiple crates increases CI time and failure surface.
- Mitigation: crate-scoped test/lint runs before any full-suite validation.

## Interfaces / Contracts
- New SDK method contract: `list_channel_messages(channel_id: &str) -> Result<Vec<ServiceApiMessage>, ServiceApiError>`.
- Updated agent-lib contract: `list_messages(channel_id: &str)` delegates to SDK list route and returns summaries.
- MCP response contract: `{"ok": true, "result": ...}` for supported operations and `{"ok": false, "error": {"kind": "unsupported_operation", ...}}` for unsupported operations.

## ADR
- Not required. No new dependency, protocol version, or architecture break introduced.
