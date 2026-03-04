# Spec: Issue 6329 - Extract service auth crypto helpers into dedicated module

## Objective
Extract service auth crypto/signature helpers from `crates/kamn-sdk/src/service.rs` into a dedicated internal module so `service.rs` stays focused on request auth envelope and API client orchestration.

## Inputs/Outputs
- Inputs:
  - Inline auth crypto/signature helpers in `service.rs`:
    - `service_signature_for_fields`
    - `service_signer_public_key_for_fields`
    - `service_signature_for_state_hash_with_private_key`
    - `service_public_key_for_private_key`
    - `service_verify_signature_with_public_key`
    - `map_service_auth_error_to_sdk`
  - Existing `kamn_core` auth helper dependencies and `SdkError` mapping semantics.
- Outputs:
  - New module file under `crates/kamn-sdk/src/` for auth crypto helper implementations.
  - `service.rs` module wiring that preserves existing public helper API from the `service` module.
  - Updated extraction contract assertions for module declaration and inline helper removal.

## Boundaries/Non-goals
- Do not change public function signatures.
- Do not change signature payload construction or verification semantics.
- Do not change error message/code mapping.
- Do not add dependencies.

## Failure modes
- Missing module wiring breaks compilation.
- Re-export mismatch changes public API surface unexpectedly.
- Error mapping drift changes `SdkError` semantics.

## Acceptance criteria (testable booleans)
- [ ] `service.rs` declares `mod service_auth_crypto;`.
- [ ] `service.rs` no longer contains inline definitions for the extracted auth helper functions and mapper.
- [ ] Public helper API remains callable from `service` module with unchanged signatures.
- [ ] `service_module_extraction_contract` includes and passes auth helper extraction assertions.
- [ ] Existing `kamn-sdk` tests for auth helper behavior remain green.

## Files to touch
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_auth_crypto.rs` (new)
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `specs/6329-service-auth-crypto-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior with existing `SdkError` variants/messages.
- Preserve mapping of `ServiceAuthSignatureError` to corresponding `SdkError` variants/fields.
- No silent fallbacks.

## Test plan
- Red phase:
  - Extend `service_module_extraction_contract` with module/inline assertions for auth helper extraction and verify failure pre-implementation.
- Green/refactor/integration phases:
  - `cargo test -p kamn-sdk --test service_module_extraction_contract`
  - `cargo test -p kamn-sdk --lib`
  - `cargo test -p kamn-sdk --test service_api_client`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
