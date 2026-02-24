# Spec: Issue #5929 - Task: Harden SDK HTTP request construction against path/header injection

- Issue: #5929
- Status: Implemented
- Type: task
- Priority: P0
- Area: sdk
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5918

## Problem Statement
User-controlled identifiers are currently interpolated directly into request paths/headers.

## Scope
In scope:
- Apply strict path-segment encoding and header-value validation (or safe client migration).

Out of scope:
- API endpoint shape changes unrelated to injection remediation.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: CRLF and delimiter injection payloads are rejected or safely encoded.
- AC-2: No direct raw interpolation of untrusted IDs into HTTP request line/headers remains.
- AC-3: Integration tests validate malformed input is fail-closed.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Regression, AC-1): `regression_service_api_client_rejects_crlf_route_identifier_payload` verifies CRLF route-segment payload is rejected fail-closed.
- C-02 (Regression, AC-1): `regression_service_request_auth_rejects_crlf_signature_payload` verifies CRLF header payload in auth signature is rejected fail-closed.
- C-03 (Functional, AC-2): `functional_service_api_client_executes_signed_http_route_contracts` verifies canonical SDK request paths/headers continue to function after sanitization.
- C-04 (Integration, AC-3): `integration_service_api_client_reads_websocket_event_frame` verifies websocket request path/header hardening does not regress websocket handshake flow.
- C-05 (Conformance, AC-4): `cargo test -p kamn-sdk --test service_api_client` passes full SDK service-client contract matrix.
- C-06 (Verify, AC-4): `cargo fmt --check` and strict `kamn-sdk` clippy pass.

## Success Metrics / Observable Signals
- Malformed route IDs and header values with control characters are rejected before request emission.
- SDK no longer interpolates unchecked dynamic route IDs or auth header values into HTTP request lines.
- Full service API SDK integration suite remains green.


## Required Test Categories
- Unit: encoder/sanitizer behavior
- Functional: SDK request builder with malicious inputs
- Integration: real HTTP server validates no injected headers
- Regression: known injection payloads blocked
- Performance: request construction non-regression

## Dependencies
- #5918
