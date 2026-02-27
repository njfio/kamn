# Spec: Issue #6129 - Configurable Service SDK timeout

Status: Accepted
Issue: #6129
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`ServiceApiClient` currently hardcodes a 2-second read/write timeout for all service endpoint sockets. This is too aggressive for production-like conditions and cannot be tuned by SDK callers.

## Scope
In scope:
- Add a configurable timeout constructor for `ServiceApiClient`.
- Keep existing `connect` behavior as backward-compatible default.
- Validate timeout input and fail closed for invalid values.
- Add unit/conformance tests for default and custom timeout behavior.
- Update SDK documentation to describe timeout configuration.

Out of scope:
- Async SDK redesign.
- Environment-variable based timeout configuration.
- Per-request timeout overrides.

## Acceptance Criteria
- AC-1: SDK exposes a public way to configure service request timeout at client construction.
- AC-2: Invalid timeout values (zero seconds) are rejected with typed `SdkError::InvalidInput`.
- AC-3: Existing `ServiceApiClient::connect` remains backward-compatible and uses default timeout behavior.
- AC-4: Configured timeout is applied to underlying TCP read/write socket timeouts.

## Conformance Cases
- C-01 (AC-1): `ServiceApiClient::connect_with_timeout_seconds("http://127.0.0.1:34052", 10)` constructs successfully.
- C-02 (AC-2): `ServiceApiClient::connect_with_timeout_seconds("http://127.0.0.1:34052", 0)` returns `InvalidInput` for `service.request_timeout_seconds`.
- C-03 (AC-3): `ServiceApiClient::connect("http://127.0.0.1:34052")` uses default timeout value (2 seconds).
- C-04 (AC-4): `ServiceEndpoint::connect_tcp_stream(Duration::from_secs(7))` produces stream read/write timeouts set to 7 seconds.

## Success Metrics
- `cargo test -p kamn-sdk --test service_api_client -- --nocapture`
- `cargo test -p kamn-sdk service::tests -- --nocapture`
- `cargo clippy -p kamn-sdk --tests -- -D warnings`
- `cargo fmt --check`
