# Plan: Issue #6129

## Approach
1. Add RED tests for configurable timeout constructor behavior and low-level timeout application.
2. Extend `ServiceApiClient` with `request_timeout_seconds` and a new constructor `connect_with_timeout_seconds`.
3. Refactor `ServiceEndpoint` connection helpers to accept timeout `Duration` from caller.
4. Keep `ServiceApiClient::connect` as a backward-compatible wrapper using the default timeout.
5. Update SDK docs with timeout configuration usage.
6. Run scoped `kamn-sdk` fmt/clippy/tests.

## Affected Modules
- `crates/kamn-sdk/src/service.rs`
- `docs/sdk/rust-sdk.md`

## Risks
- Risk: constructor-surface change can unintentionally break call sites.
  - Mitigation: preserve existing `connect` signature and behavior.
- Risk: timeout validation reason codes drift.
  - Mitigation: lock with explicit conformance tests.

## Interfaces/Contracts
- New public API: `ServiceApiClient::connect_with_timeout_seconds(endpoint, timeout_seconds)`.
- Existing API preserved: `ServiceApiClient::connect(endpoint)`.
