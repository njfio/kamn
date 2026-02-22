# Spec: #5665 Activate MCP Server Execution and List-Messages Parity

- Issue: #5665
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-mcp-server` currently returns static readiness output and does not execute tool invocations. `kamn-agent-lib` still returns `UnsupportedOperation` for `list_messages` despite an existing service route for channel message listing.

## Scope
### In Scope
- Add `kamn-sdk` client support for channel message listing via service API route `/v1/channels/{id}/messages`.
- Implement `KamnAgentHandle::list_messages` in `kamn-agent-lib` using the SDK client path.
- Implement request parsing and tool dispatch execution in `kamn-mcp-server` for currently supported operations.
- Return explicit structured unsupported responses for operations without service support.
- Add spec-derived unit/functional/integration/regression tests.

### Out of Scope
- New service API routes for `accept_task`, `complete_task`, `fund_escrow`, or `release_escrow`.
- Scenario orchestration coverage in `kamn-e2e-harness`.
- CI/workflow or shell-surface changes.

## Acceptance Criteria
### AC-1 SDK list-messages route support
Given a valid channel id,
When `ServiceApiClient` requests channel messages,
Then it performs GET `/v1/channels/{id}/messages` and decodes message summary records.

### AC-2 Agent-lib list-messages implementation
Given an initialized `KamnAgentHandle` and channel id,
When `list_messages` is called,
Then it returns message summaries instead of `UnsupportedOperation`.

### AC-3 MCP supported tool execution
Given a valid MCP tool invocation for a supported operation,
When `kamn-mcp-server` receives the request,
Then it dispatches through `kamn-agent-lib` and returns structured success JSON.

### AC-4 MCP unsupported tool contract
Given a valid MCP tool invocation for an operation not yet service-supported,
When `kamn-mcp-server` receives the request,
Then it returns a structured unsupported response identifying the operation and reason.

### AC-5 Regression safety for prior stubs
Given regression tests for the previous `list_messages` unsupported behavior,
When the new implementation is applied,
Then tests demonstrate the former stub path is replaced by passing behavior without breaking existing supported operations.

## Conformance Cases
- C-01 (AC-1): SDK route test verifies GET `/v1/channels/{id}/messages` path construction and response decoding.
- C-02 (AC-2, AC-5): Agent-lib test verifies `list_messages` returns channel message summaries and no longer yields `UnsupportedOperation`.
- C-03 (AC-3): MCP dispatch test verifies supported operation request produces success payload with expected fields.
- C-04 (AC-4): MCP dispatch test verifies unsupported operation request produces deterministic structured error payload.

## Success Metrics
- `cargo test -p kamn-sdk` passes with new list-messages coverage.
- `cargo test -p kamn-agent-lib` passes with regression coverage for `list_messages`.
- `cargo test -p kamn-mcp-server` passes with tool dispatch conformance coverage.
- `cargo fmt --all --check` and `cargo clippy -- -D warnings` pass for touched crates.
