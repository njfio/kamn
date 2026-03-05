# Spec: Issue 6453 - Extract signer signature verification test cases

## Objective
Extract baseline-compatibility and local-signature verification scenarios from `crates/kamn-core/tests/signer_backend.rs` into a dedicated `signer_signature_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline signature verification tests in `signer_backend.rs`.
- Outputs:
  - New `crates/kamn-core/tests/signer_backend/signer_signature_cases.rs` containing extracted signature verification scenario bodies.
  - Root wrappers that keep original test names and delegate to `signer_signature_cases`.
  - Updated `crates/kamn-core/tests/signer_backend_split_contract.rs` with signature-case delegation/ownership markers.

## Boundaries/Non-goals
- No behavior changes to baseline-v1 compatibility gating or signature verification semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline signature verification test bodies.
- Split-contract guard fails to enforce signature-case delegation markers.
- Extracted tests lose required fail-closed assertions for verification regressions.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `signer_signature_cases.rs` exists and contains extracted signature verification scenario bodies.
- [ ] AC-2: root `signer_backend.rs` retains selected entrypoint names and delegates to `signer_signature_cases` functions.
- [ ] AC-3: `signer_backend_split_contract.rs` enforces signature delegation and case ownership markers.
- [ ] AC-4: `cargo test -p kamn-core --test signer_backend_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6453-signer-signature-verification-test-cases-extraction.md`
- `crates/kamn-core/tests/signer_backend.rs`
- `crates/kamn-core/tests/signer_backend/signer_signature_cases.rs` (new)
- `crates/kamn-core/tests/signer_backend_split_contract.rs`

## Error semantics
- Preserve typed assertion semantics for verification regression paths.
- Preserve panic/fail-loud semantics around malformed env/config usage.
- Preserve assertion diagnostics for CI debuggability.

## Test plan
- Red:
  - Add signature marker expectations to `signer_backend_split_contract.rs` before wiring module; verify failing split-contract test.
- Green:
  - Move selected signature verification test bodies to `signer_signature_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate signature request/test literals and helper setup without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
