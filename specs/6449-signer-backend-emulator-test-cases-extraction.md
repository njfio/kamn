# Spec: Issue 6449 - Extract signer backend emulator test cases

## Objective
Extract signer-emulator budget/performance scenarios from `crates/kamn-core/tests/signer_backend.rs` into a dedicated `signer_emulator_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline signer-emulator tests in `signer_backend.rs`.
- Outputs:
  - New `crates/kamn-core/tests/signer_backend/signer_emulator_cases.rs` containing extracted signer-emulator scenario bodies.
  - Root wrappers that keep original test names and delegate to `signer_emulator_cases`.
  - New split-contract guard `crates/kamn-core/tests/signer_backend_split_contract.rs` enforcing delegation/ownership markers.

## Boundaries/Non-goals
- No behavior changes to signer emulator budget parser/comparator semantics or performance-lane logic.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline signer-emulator test bodies.
- Split-contract guard fails to enforce extracted ownership markers.
- Extracted tests lose required env-lock/fail-loud behavior.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `signer_emulator_cases.rs` exists and contains extracted signer-emulator scenario bodies.
- [ ] AC-2: root `signer_backend.rs` retains entrypoint names and delegates to `signer_emulator_cases` functions.
- [ ] AC-3: `signer_backend_split_contract.rs` enforces delegation and case ownership markers.
- [ ] AC-4: `cargo test -p kamn-core --test signer_backend_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6449-signer-backend-emulator-test-cases-extraction.md`
- `crates/kamn-core/tests/signer_backend.rs`
- `crates/kamn-core/tests/signer_backend/signer_emulator_cases.rs` (new)
- `crates/kamn-core/tests/signer_backend_split_contract.rs` (new)

## Error semantics
- Preserve existing fail-loud panic behavior for invalid budget override parsing.
- Preserve assertion semantics for budget boundaries and backend selection.

## Test plan
- Red:
  - Add split-contract guard markers for signer-emulator extraction before wiring module; verify failing test.
- Green:
  - Move signer-emulator tests to `signer_emulator_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate repeated literals in extracted module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test signer_backend_split_contract`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
