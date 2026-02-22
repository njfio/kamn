# Plan: #5668 Activate MCP Verify-Proof Tool via Agent-Lib

## Approach
1. Add RED tests for verify-proof success and malformed-input contracts in `tool_dispatch_contract.rs`.
2. Extend `McpToolBackend` and `dispatch_tool_request_json` to support verify-proof payload handling.
3. Keep unsupported-operation mapping unchanged for non-implemented task/escrow operations.
4. Run targeted `kamn-mcp-server` test/lint/format gates.

## Affected Modules
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-mcp-server/tests/tool_dispatch_contract.rs`

## Risks and Mitigations
- Risk: malformed request parsing could silently map to backend errors.
- Mitigation: explicit field checks and deterministic `invalid_request` envelopes in dispatcher.
- Risk: unsupported operations could regress during dispatch refactor.
- Mitigation: retain explicit unsupported mapping and regression assertions.

## Interfaces / Contracts
- Added backend trait method: `verify_proof(message_id, tx_hash, block_height, finality)`.
- Dispatcher contract: valid verify-proof request yields `ok:true` result; malformed request yields `ok:false` + `invalid_request`; unimplemented operations remain `unsupported_operation`.

## ADR
- Not required. No architecture or dependency changes.
