# Spec: Issue 6331 - Extract ServiceRequestAuth internals into dedicated module

## Objective
Extract `ServiceRequestAuth` model and constructor validation logic from `crates/kamn-sdk/src/service.rs` into a dedicated internal module, preserving existing API and validation/error semantics.

## Inputs/Outputs
- Inputs:
  - Inline `ServiceRequestAuth` struct and impl in `service.rs`.
  - Existing header validation helper (`validate_http_header_value`) and `SdkError` semantics.
- Outputs:
  - New module under `crates/kamn-sdk/src/` containing `ServiceRequestAuth` and its impl.
  - `service.rs` updated to wire/import the request-auth module.
  - Extraction contract tests updated to assert module declaration and inline removal.

## Boundaries/Non-goals
- Do not change public API signatures for `ServiceRequestAuth` constructors.
- Do not change request auth validation behavior or error field/reason strings.
- Do not change signing/verification helper behavior.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile failures.
- Visibility mismatch breaks existing call sites.
- Behavior drift in validation changes `SdkError` mapping.

## Acceptance criteria (testable booleans)
- [x] `service.rs` declares `mod service_request_auth;`.
- [x] `service.rs` no longer contains inline `ServiceRequestAuth` struct/impl.
- [x] Existing constructor behavior/signatures remain unchanged.
- [x] `service_module_extraction_contract` includes and passes request-auth extraction assertions.
- [x] Existing `kamn-sdk` tests covering request-auth behavior remain green.

## Files to touch
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_request_auth.rs` (new)
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `specs/6331-service-request-auth-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior and existing `SdkError::InvalidInput` field/reason values.
- No silent fallback paths.

## Test plan
- Red phase:
  - Extend `service_module_extraction_contract` to assert request-auth module declaration and inline removal; confirm failures before implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-sdk --test service_module_extraction_contract`
  - `cargo test -p kamn-sdk --lib`
  - `cargo test -p kamn-sdk --test service_api_client`

## Phase 6 integration evidence
- `cargo test -p kamn-sdk --test service_module_extraction_contract`:
  - pass (`10 passed, 0 failed`)
- `cargo test -p kamn-sdk --lib`:
  - pass (`21 passed, 0 failed`)
- `cargo test -p kamn-sdk --test service_api_client`:
  - pass (`15 passed, 0 failed`)

## Deviations
- None.
