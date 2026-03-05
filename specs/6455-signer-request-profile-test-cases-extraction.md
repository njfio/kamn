# Spec: Issue 6455 - Extract signer request/profile contract test cases

## Objective
Extract request/profile contract scenarios from `crates/kamn-core/tests/signer_backend.rs` into a dedicated `signer_request_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline request/profile tests in `signer_backend.rs`.
- Outputs:
  - New `crates/kamn-core/tests/signer_backend/signer_request_cases.rs` containing extracted request/profile scenario bodies.
  - Root wrappers that keep original test names and delegate to `signer_request_cases`.
  - Updated `crates/kamn-core/tests/signer_backend_split_contract.rs` with request/profile delegation and ownership markers.

## Boundaries/Non-goals
- No behavior changes to `SigningRequest::for_transaction` validation or signature-profile assertions.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline request/profile test bodies.
- Split-contract guard fails to enforce request/profile delegation markers.
- Extracted tests lose assertion diagnostics for profile-prefix expectations.

## Acceptance criteria (testable booleans)
- [x] AC-1: `signer_request_cases.rs` exists and contains extracted request/profile scenario bodies.
- [x] AC-2: root `signer_backend.rs` retains selected entrypoint names and delegates to `signer_request_cases` functions.
- [x] AC-3: `signer_backend_split_contract.rs` enforces request/profile delegation and case ownership markers.
- [x] AC-4: `cargo test -p kamn-core --test signer_backend_split_contract` passes.
- [x] AC-5: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6455-signer-request-profile-test-cases-extraction.md`
- `crates/kamn-core/tests/signer_backend.rs`
- `crates/kamn-core/tests/signer_backend/signer_request_cases.rs` (new)
- `crates/kamn-core/tests/signer_backend_split_contract.rs`

## Error semantics
- Preserve typed `EmptyField("transaction_id")` assertion behavior.
- Preserve fail-loud signature/profile assertions and diagnostic messages.

## Test plan
- Red:
  - Add request/profile marker expectations to `signer_backend_split_contract.rs` before wiring module; verify failing split-contract test.
- Green:
  - Move selected request/profile test bodies to `signer_request_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate repeated request/profile literals and helper setup without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Split contract guard:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - result: `4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- Full signer backend integration target:
  - `cargo test -p kamn-core --test signer_backend`
  - result: `30 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`

## Deviations
- None.
