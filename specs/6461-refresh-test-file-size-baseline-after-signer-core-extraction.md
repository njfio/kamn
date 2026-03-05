# Spec: Issue 6461 - Refresh test file size baseline after signer core extraction

## Objective
Refresh `test_file_size_policy` baseline inventory counts after signer core-case extraction so policy assertions match the current intended test inventory.

## Inputs/Outputs
- Inputs:
  - Existing policy test `crates/kamn-core/tests/test_file_size_policy.rs`.
  - Existing baseline fixture `fixtures/ci/test_file_size_policy_baseline.env`.
- Outputs:
  - Updated baseline fixture values consistent with current repository test inventory.
  - Passing `cargo test -p kamn-core --test test_file_size_policy`.

## Boundaries/Non-goals
- No policy threshold changes.
- No policy assertion removals.
- No production code or workflow changes.

## Failure modes
- Baseline values still drift from computed inventory values.
- Policy test passes by weakening checks instead of refreshing baseline data.
- Fix regresses signer backend test target.

## Acceptance criteria (testable booleans)
- [ ] AC-1: local run reproduces drift assertion (`left: 429`, `right: 428`).
- [ ] AC-2: baseline fixture is updated to current inventory counts.
- [ ] AC-3: `cargo test -p kamn-core --test test_file_size_policy` passes.
- [ ] AC-4: `cargo test -p kamn-core --test signer_backend` passes.

## Files to touch
- `specs/6461-refresh-test-file-size-baseline-after-signer-core-extraction.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error semantics
- Preserve fail-loud assertion behavior in `test_file_size_policy.rs`.
- Preserve baseline/threshold schema validation behavior.

## Test plan
- Red:
  - Run `cargo test -p kamn-core --test test_file_size_policy` and confirm drift failure.
- Green:
  - Update baseline fixture counts to current computed values.
- Refactor:
  - Keep baseline fixture readability and explanatory comments consistent.
- Integration:
  - `cargo test -p kamn-core --test test_file_size_policy`
  - `cargo test -p kamn-core --test signer_backend`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
