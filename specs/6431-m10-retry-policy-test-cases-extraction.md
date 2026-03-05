# Spec: Issue 6431 - Extract M10 archival retry policy test cases

## Objective
Extract archival retry policy scenario tests (`spec_c12` through `spec_c15`) from `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` into a dedicated `retry_policy_cases` module while preserving root test entrypoints and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline C12-C15 tests in `data_layer_m10_partition_archival.rs`.
  - Existing split-contract guard in `data_layer_m10_partition_archival_split_contract.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m10_partition_archival/retry_policy_cases.rs` containing C12-C15 scenario bodies.
  - Root contract tests for C12-C15 that delegate to `retry_policy_cases`.
  - Split-contract guard coverage for the new delegation markers.

## Boundaries/Non-goals
- No behavior changes to retry policy projections or error semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline C12-C15 scenario logic after extraction.
- Split contract does not enforce delegation markers for C12-C15.
- Refactor accidentally changes assertion expectations or reason-code checks.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `retry_policy_cases.rs` exists and contains C12-C15 scenario bodies.
- [ ] AC-2: root `data_layer_m10_partition_archival.rs` retains C12-C15 entrypoints and delegates to `retry_policy_cases` functions.
- [ ] AC-3: split-contract test enforces C12-C15 delegation markers.
- [ ] AC-4: `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test data_layer_m10_partition_archival` passes.

## Files to touch
- `specs/6431-m10-retry-policy-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival/retry_policy_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m10_partition_archival_split_contract.rs`

## Error semantics
- Preserve fail-closed assertions and reason-code checks in C12-C15.
- Preserve existing panic/expect messages where behavior is unchanged.

## Test plan
- Red:
  - Add split-contract markers for C12-C15 delegation and cases-module ownership before adding module wiring; verify failing test.
- Green:
  - Wire root C12-C15 tests to delegated functions and move bodies into `retry_policy_cases.rs`.
- Refactor:
  - Reduce duplication and tighten constants in the extracted module without behavior change.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival_split_contract`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
