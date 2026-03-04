# Spec: Issue 6335 - Extract ServiceApiClient orchestration into dedicated module

## Objective
Move `ServiceApiClient` orchestration implementation out of `crates/kamn-sdk/src/service.rs` into a dedicated internal module while preserving public API and behavior.

## Inputs/Outputs
- Inputs:
  - Inline definitions in `service.rs`:
    - `struct HttpResponse`
    - `pub struct ServiceApiClient`
    - `impl ServiceApiClient`
    - `parse_escrow_status` helper
  - Existing helper modules/constants used by the client implementation.
- Outputs:
  - New module file `service_client.rs` containing extracted client orchestration code.
  - `service.rs` module wiring and re-export for `ServiceApiClient`.
  - Contract updates for module declaration and inline-removal assertions.

## Boundaries/Non-goals
- No public method signature changes for `ServiceApiClient`.
- No route behavior/response parsing semantic changes.
- No error mapping semantic changes.
- No new dependencies.

## Failure modes
- Visibility/import mismatch breaks compilation.
- Behavior drift in request execution or route methods.
- Missing re-export breaks external import paths.

## Acceptance criteria (testable booleans)
- [ ] `service.rs` declares `mod service_client;`.
- [ ] `service.rs` no longer contains inline `struct HttpResponse`, `pub struct ServiceApiClient`, or `impl ServiceApiClient`.
- [ ] `ServiceApiClient` remains publicly reachable through `kamn_sdk::service`.
- [ ] `service_module_extraction_contract` includes and passes client extraction assertions.
- [ ] Existing `kamn-sdk` client behavior tests remain green.

## Files to touch
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_client.rs` (new)
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `specs/6335-service-client-module-extraction.md`

## Error semantics
- Preserve current hard-fail behavior and `SdkError` outcomes for request execution paths.

## Test plan
- Red phase:
  - Extend extraction contract with service client module/inline assertions and confirm failure.
- Green/refactor/integration phases:
  - `cargo test -p kamn-sdk --test service_module_extraction_contract`
  - `cargo test -p kamn-sdk --lib`
  - `cargo test -p kamn-sdk --test service_api_client`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
