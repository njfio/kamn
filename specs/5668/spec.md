# Spec: #5668 Activate MCP Verify-Proof Tool via Agent-Lib

- Issue: #5668
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-mcp-server` still reports `verify_proof` as unsupported even though `kamn-agent-lib` already supports deterministic Kolme proof verification.

## Scope
### In Scope
- Add `verify_proof` to MCP backend trait and dispatch implementation.
- Parse and validate deterministic verify-proof request fields.
- Return structured success payload for valid proof verification.
- Return deterministic `invalid_request` payload for malformed verify-proof inputs.
- Preserve explicit unsupported responses for operations lacking service route support.

### Out of Scope
- New Kolme transport/RPC endpoints.
- Service API route additions.
- Any shell/workflow/template changes.

## Acceptance Criteria
### AC-1 Verify-proof tool dispatch wiring
Given a valid `verify_proof` tool request,
When MCP dispatch executes,
Then it calls `KamnAgentHandle::verify_proof` through backend abstraction.

### AC-2 Verify-proof success payload contract
Given a valid proof receipt and message id,
When MCP dispatch runs `verify_proof`,
Then it returns a structured success payload including `message_id`, `block_height`, `finality`, and `verified`.

### AC-3 Verify-proof invalid-request contract
Given a malformed verify-proof request missing required fields or invalid numeric values,
When MCP dispatch runs,
Then it returns deterministic `invalid_request` error payload.

### AC-4 Unsupported-op contract preservation
Given currently unsupported operations (`accept_task`, `complete_task`, `fund_escrow`, `release_escrow`),
When MCP dispatch runs,
Then it still returns deterministic `unsupported_operation` payloads.

## Conformance Cases
- C-01 (AC-1, AC-2): dispatching valid `verify_proof` request yields success payload with normalized finality/verified projection.
- C-02 (AC-3): dispatching malformed `verify_proof` request yields deterministic `invalid_request` payload.
- C-03 (AC-4): dispatching unsupported operation still yields deterministic `unsupported_operation` payload.

## Success Metrics
- `cargo test -p kamn-mcp-server --test tool_dispatch_contract` passes with verify-proof conformance coverage.
- `cargo test -p kamn-mcp-server` passes without regressions.
- `cargo fmt --all --check` and `cargo clippy -p kamn-mcp-server -- -D warnings` pass.
