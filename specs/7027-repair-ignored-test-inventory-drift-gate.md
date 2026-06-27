# 7027-repair-ignored-test-inventory-drift-gate

## Objective
Restore the Fast Gate ignored-test inventory drift check by refreshing the
deterministic ignored-test baseline and metadata fixtures to match the current
repo inventory without weakening ignored-test policy.

## Inputs/Outputs
- Inputs:
  - `scripts/ci/generate_ignored_test_inventory_baseline.sh`
  - `scripts/ci/check_ignored_test_inventory_drift.sh`
  - `fixtures/ci/ignored_test_inventory_baseline.json`
  - `fixtures/ci/ignored_test_inventory_metadata.json`
  - `fixtures/ci/ignored_test_promotion_criteria.json`
- Outputs:
  - `bash scripts/ci/test_check_ignored_test_inventory_drift.sh` passes.
  - `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh`
    passes.
  - `bash scripts/ci/test_ignored_test_inventory_parser_contract.sh` passes.
  - Baseline count matches the generated deterministic inventory.

## Boundaries/Non-goals
- Do not remove ignored-test inventory checks from Fast Gate.
- Do not weaken metadata, promotion-criteria, stale-rationale, or drift
  failure semantics.
- Do not change ignored test annotations in product code under this issue.
- Do not broaden this issue into MVP demo feature work.

## Failure Modes
- Baseline fixture drifts from generated current inventory and fails closed.
- Metadata fixture contains stale moved-path entries after test extraction.
- New ignored live/local-runtime tests lack owner, reason, disposition, or
  tracking issue metadata.
- Promotion criteria do not cover a metadata reason category.

## Acceptance Criteria
- [x] Red evidence captures `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
      failing with `ignored-test generated baseline drift detected`.
- [x] Generated ignored-test inventory count is deterministic and recorded.
- [x] Baseline fixture matches generated ignored-test inventory.
- [x] Metadata fixture covers every baseline entry and has no stale entries.
- [x] Metadata policy and parser contracts pass.
- [x] `cargo fmt --check`, strict workspace clippy, and `make check` remain
      green.

## Files To Touch
- `fixtures/ci/ignored_test_inventory_baseline.json`
- `fixtures/ci/ignored_test_inventory_metadata.json`
- `scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh`
- A Rust contract test under `crates/kamn-core/tests/`

## Error Semantics
- Baseline drift remains a hard failure.
- Missing metadata remains a hard failure.
- Stale metadata remains a hard failure.
- Missing promotion criteria remain a hard failure.

## Test Plan
- Red: run `bash scripts/ci/test_check_ignored_test_inventory_drift.sh` and
  capture the generated baseline drift failure.
- Red: add a Rust contract that requires the refreshed generated count and
  observe it fail before fixture refresh.
- Green: refresh baseline and metadata fixtures from deterministic generated
  inventory and explicit metadata decisions.
- Integration: rerun drift, metadata policy, and parser contracts.

## Completion Evidence
- Red: `bash scripts/ci/test_check_ignored_test_inventory_drift.sh` failed with
  `ignored-test generated baseline drift detected`.
- Red: `cargo test -p kamn-core --test ignored_test_inventory_fixture_contract`
  failed because the baseline still had `ignored_test_count=11`.
- Green: generator output reported `ignored_test_count=18`,
  `unresolved_marker_count=0`, and `reason_codes=none`.
- Green: `cargo test -p kamn-core --test ignored_test_inventory_fixture_contract`
  passed.
- Green: `bash scripts/ci/test_check_ignored_test_inventory_drift.sh` passed.
- Green: `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh`
  passed.
- Green: `bash scripts/ci/test_ignored_test_inventory_parser_contract.sh`
  passed.
- Closeout: `cargo fmt --check` passed.
- Closeout: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.
- Closeout: `make check` passed.
- Closeout: governance ratio passed with `governance_commit_count=6`,
  `feature_commit_count=44`, and `governance_ratio=0.12`.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +20`
- `rust_loc_delta_estimate: +120`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7027`
- `shell_loc_delta_actual: +222`
- `rust_loc_delta_actual: +166366`
- `shell_to_rust_ratio_delta_actual: -0.181631`
- `shell_surface_ratio_target_status: improved`
