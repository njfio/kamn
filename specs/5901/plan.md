# Plan: Issue #5901 - Replace Fake SDK Live Transport with Network-Backed Behavior

## Approach
1. Replace `LiveTransportKamnClient` internals with:
   - `ServiceApiClient` for HTTP route calls.
   - deterministic chain/signature context used to build Service API auth headers.
   - bounded internal ID maps only for converting service string IDs to SDK numeric wrappers where required by current SDK type surface.
2. Implement network-backed methods:
   - `send` via `ServiceApiClient::send_message`.
   - `resolve` and `get_reputation` via `ServiceApiClient::get_agent_profile`.
3. Mark unsupported methods as explicit `SdkError::NotImplemented` (no in-memory fallback).
4. Replace live transport tests that currently assume shared endpoint in-memory state with loopback server integration tests.

## Affected Modules
- `crates/kamn-sdk/src/live.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`

## Risks / Mitigations
- Risk: Service API auth scope/signature mismatch in tests.
  - Mitigation: reuse service auth signing helpers and deterministic test key env setup.
- Risk: message ID conversion mismatch due SDK `MessageId(u64)` type.
  - Mitigation: deterministic, collision-checked string->u64 mapping with explicit conflict error.
- Risk: flaky socket tests.
  - Mitigation: bind loopback ephemeral ports, readiness wait loop, fixed request budgets.

## Interfaces / Contracts
- No public trait signature changes in this issue.
- Behavior contract change: unsupported methods now fail closed with `SdkError::NotImplemented` in live mode.

## ADR
- Not required for this scoped remediation (no new dependency, protocol change, or cross-crate architecture decision).
