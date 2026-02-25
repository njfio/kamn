# Spec: Issue #5930 - Task: Implement HTTPS support in SDK service client and TLS validation

- Issue: #5930
- Status: Implemented
- Type: task
- Priority: P1
- Area: sdk
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5918

## Problem Statement
HTTPS is currently explicitly unsupported, forcing plaintext traffic.

## Scope
In scope:
- Implement HTTPS scheme support with certificate validation and clear configuration controls.

Out of scope:
- Mutual-TLS feature expansion not required for this task.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: SDK supports HTTPS endpoints with strict cert validation.
- AC-2: TLS misconfiguration and invalid cert chains fail closed with deterministic errors.
- AC-3: Integration tests with fixture certs pass.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Integration, AC-1/AC-3): `spec_c01_service_api_client_executes_https_health_route_with_trusted_ca` verifies HTTPS health-route success with explicit trusted CA fixture chain.
- C-02 (Regression, AC-2): `spec_c02_service_api_client_rejects_untrusted_https_certificate_chain` verifies fail-closed certificate verification rejection when CA is not trusted.
- C-03 (Regression, AC-2): `spec_c02_service_api_client_rejects_missing_tls_ca_bundle_path` verifies deterministic fail-closed error for missing custom CA bundle path.
- C-04 (Conformance, AC-4): `cargo test -p kamn-sdk --test service_api_client` passes full service-client route contract suite.
- C-05 (Verify, AC-4): `cargo test -p kamn-sdk`, `cargo clippy -p kamn-sdk -- -D warnings`, and `cargo fmt --check` pass.

## Success Metrics / Observable Signals
- HTTPS endpoints execute successfully through rustls-backed transport when the certificate chain is trusted.
- TLS certificate trust failures and CA-file misconfiguration return deterministic `SdkError::TransportFailure`/`SdkError::InvalidInput` results.
- Fixture-backed HTTPS integration tests and full `kamn-sdk` test suite pass.


## Required Test Categories
- Unit: scheme/config validation
- Functional: HTTPS request path
- Integration: HTTPS local fixture server
- Regression: NotImplemented HTTPS path removed
- Performance: connection overhead baseline documented

## Dependencies
- #5918
