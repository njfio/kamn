# Spec: Issue 6327 - Extract service endpoint/transport internals into dedicated module

## Objective
Extract endpoint parsing and transport bootstrap internals from `crates/kamn-sdk/src/service.rs` into a dedicated internal module to reduce coupling between transport setup and service API orchestration.

## Inputs/Outputs
- Inputs:
  - Inline endpoint/transport internals currently in `service.rs`:
    - `ServiceScheme`
    - `ServiceStream`
    - `ServiceEndpoint`
    - `resolve_tls_client_config`
    - `resolve_request_timeout_seconds`
    - `resolve_tls_server_name`
  - Existing constants and `SdkError` behavior used by those internals.
- Outputs:
  - New module file under `crates/kamn-sdk/src/` with extracted endpoint/transport internals.
  - `service.rs` module wiring/imports updated so existing call sites and tests continue to work.
  - Extraction contract test updates asserting module declaration and inline removal.

## Boundaries/Non-goals
- Do not change public SDK API signatures.
- Do not change request/response/auth semantics.
- Do not change timeout/TLS failure semantics.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Visibility mismatch breaks `service_tests.rs` imports.
- Transport behavior drift changes endpoint parsing or TLS/timeout error semantics.

## Acceptance criteria (testable booleans)
- [x] `service.rs` declares `mod service_endpoint;`.
- [x] `service.rs` no longer contains inline definitions for:
  - `enum ServiceScheme`
  - `enum ServiceStream`
  - `struct ServiceEndpoint`
  - `resolve_tls_client_config`
  - `resolve_request_timeout_seconds`
  - `resolve_tls_server_name`
- [x] Existing `kamn-sdk` tests for service timeout and endpoint transport paths remain green.
- [x] `service_module_extraction_contract` includes and passes endpoint/transport extraction assertions.

## Files to touch
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_endpoint.rs` (new)
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `specs/6327-service-endpoint-transport-module-extraction.md`

## Error semantics
- Preserve existing hard-fail behavior and current `SdkError` variants/messages.
- Preserve TLS CA bundle validation behavior and TLS handshake/certificate mapping.
- Preserve timeout environment parsing semantics.

## Test plan
- Red phase:
  - Extend `service_module_extraction_contract` with module/inline assertions for endpoint/transport extraction; confirm failure before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-sdk --test service_module_extraction_contract`
  - `cargo test -p kamn-sdk --lib`
  - `cargo test -p kamn-sdk --test service_api_client`

## Phase 6 integration evidence
- `cargo test -p kamn-sdk --test service_module_extraction_contract`:
  - pass (`6 passed, 0 failed`)
- `cargo test -p kamn-sdk --lib`:
  - pass (`21 passed, 0 failed`)
- `cargo test -p kamn-sdk --test service_api_client`:
  - pass (`15 passed, 0 failed`)

## Deviations
- None.
