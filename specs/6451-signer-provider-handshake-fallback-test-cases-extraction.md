# Spec: Issue 6451 - Extract signer provider handshake/fallback test cases

## Objective
Extract provider-handshake and fallback-policy scenarios from `crates/kamn-core/tests/signer_backend.rs` into a dedicated `signer_provider_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline provider-handshake/fallback tests in `signer_backend.rs`.
- Outputs:
  - New `crates/kamn-core/tests/signer_backend/signer_provider_cases.rs` containing extracted provider-handshake/fallback scenario bodies.
  - Root wrappers that keep original test names and delegate to `signer_provider_cases`.
  - Updated `crates/kamn-core/tests/signer_backend_split_contract.rs` with provider-case delegation/ownership markers.

## Boundaries/Non-goals
- No behavior changes to signer routing, provider handshake, role-policy fallback, or backend-mismatch semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline provider-handshake/fallback test bodies.
- Split-contract guard fails to enforce provider delegation markers.
- Extracted tests lose expected fail-closed assertions for provider/fallback error cases.

## Acceptance criteria (testable booleans)
- [x] AC-1: `signer_provider_cases.rs` exists and contains extracted provider-handshake/fallback scenario bodies.
- [x] AC-2: root `signer_backend.rs` retains selected entrypoint names and delegates to `signer_provider_cases` functions.
- [x] AC-3: `signer_backend_split_contract.rs` enforces provider delegation and case ownership markers.
- [x] AC-4: `cargo test -p kamn-core --test signer_backend_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6451-signer-provider-handshake-fallback-test-cases-extraction.md`
- `crates/kamn-core/tests/signer_backend.rs`
- `crates/kamn-core/tests/signer_backend/signer_provider_cases.rs` (new)
- `crates/kamn-core/tests/signer_backend_split_contract.rs`

## Error semantics
- Preserve typed error assertions for:
  - `FallbackDeniedByRolePolicy`
  - `UnsupportedSecureProvider`
  - `ProviderHandshakeRejected`
  - `ProviderClientBackendMismatch`
  - `SecureProviderBackendMismatch`
- Preserve current assertion diagnostics for CI debuggability.

## Test plan
- Red:
  - Add provider marker expectations to `signer_backend_split_contract.rs` before wiring module; verify failing split-contract test.
- Green:
  - Move selected provider-handshake/fallback test bodies to `signer_provider_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate provider request literals and handshake-matrix setup helpers without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Split contract guard:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - result: `2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- Full signer backend integration target:
  - `cargo test -p kamn-core --test signer_backend`
  - result: `30 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`

## Deviations
- None.
