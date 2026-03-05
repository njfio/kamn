# Spec: Issue 6458 - Fix test file size policy inventory drift

## Objective
Restore `ci-fast-gate` by updating `test_file_size_policy` baseline inventory data to match the current intended test-file inventory without weakening policy enforcement.

## Inputs/Outputs
- Inputs:
  - Existing policy test `crates/kamn-core/tests/test_file_size_policy.rs`.
  - Existing baseline fixture `fixtures/ci/test_file_size_policy_baseline.env`.
- Outputs:
  - Updated baseline fixture values that align with current inventory counts.
  - Passing `cargo test -p kamn-core --test test_file_size_policy`.

## Boundaries/Non-goals
- No policy check removal or threshold loosening.
- No production source changes.
- No CI workflow changes.

## Failure modes
- Baseline values still drift from computed inventory counts.
- Policy test passes by weakening assertions instead of updating baseline truth data.
- Fix resolves inventory drift but regresses related signer backend tests.

## Acceptance criteria (testable booleans)
- [x] AC-1: local run reproduces pre-fix failing `test_file_size_policy` drift assertion.
- [x] AC-2: baseline fixture values are updated to match current intended inventory counts.
- [x] AC-3: `cargo test -p kamn-core --test test_file_size_policy` passes.
- [x] AC-4: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6458-fix-test-file-size-policy-inventory-drift.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error semantics
- Preserve existing fail-loud assertion behavior in `test_file_size_policy.rs`.
- Preserve schema/version validation behavior for threshold and baseline files.

## Test plan
- Red:
  - Run `cargo test -p kamn-core --test test_file_size_policy` and capture failing drift assertion.
- Green:
  - Update baseline fixture values to match current computed counts.
- Refactor:
  - Keep fixture ordering/readability consistent; no behavior changes.
- Integration:
  - `cargo test -p kamn-core --test test_file_size_policy`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Red reproduction:
  - `cargo test -p kamn-core --test test_file_size_policy`
  - failure: `spec_c04_oversized_test_counts_are_within_budget` with `test file inventory drift` (`left: 428`, `right: 398`)
- Post-fix verification:
  - `cargo test -p kamn-core --test test_file_size_policy`
  - result: `4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
  - `cargo test -p kamn-core --test signer_backend`
  - result: `30 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`

## Deviations
- None.
