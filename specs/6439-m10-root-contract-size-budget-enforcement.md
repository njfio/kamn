# Spec: Issue 6439 - Enforce M10 root archival contract size budget

## Objective
Bring `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` to `<=200` LOC while preserving delegated test behavior and entrypoint naming.

## Inputs/Outputs
- Inputs:
  - Existing root archival contract file at 215 LOC.
  - Existing split-contract ownership guard.
- Outputs:
  - Root contract file at `<=200` LOC.
  - Split-contract test that enforces the size budget.

## Boundaries/Non-goals
- No behavior changes to C01-C38 scenario execution.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Line-count reduction accidentally removes entrypoint names or delegation mapping.
- Split-contract budget assertion drifts from repo policy and allows regression.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `data_layer_m10_partition_archival.rs` line count is `<= 200`.
- [ ] AC-2: C01-C38 entrypoint names remain present and delegate to same case runners.
- [ ] AC-3: split-contract suite checks and enforces the `<= 200` line budget.
- [ ] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch
- `specs/6439-m10-root-contract-size-budget-enforcement.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics
- No assertion or reason-code behavior changes.

## Test plan
- Red:
  - Add split-contract size-budget assertion (`<= 200`) and verify failure at current line count.
- Green:
  - Reduce root file LOC through structural deduplication of delegation wrappers while preserving naming and mapping.
- Refactor:
  - Keep delegation macro/readability clear without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
