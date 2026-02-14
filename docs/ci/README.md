# CI Docs Index

This directory tracks CI policy, cost controls, and contract-lane strategy.

- Primary policy guide: `docs/ci/ci-cost-and-lane-framework.md`
- CI routing and selector contracts: `docs/ci/strategy.md`
- Billing closeout runbook: `docs/ci/post-billing-closeout.md`

## Kolme Wrapper Trend Guard Reason Codes

`scripts/ci/check_kolme_wrapper_budget_trend.sh` is backed by
`scripts/ci/kolme_wrapper_inventory_baseline.py` in `--trend-mode`.
Fail-closed reason codes are deterministic and include:

- `wrapper_count_delta_threshold_exceeded`
- `total_shell_loc_delta_threshold_exceeded`
- `lane_shell_loc_increase_violation`
- `unexpected_new_lanes_in_current_inventory`
- `baseline_wrapper_count_invalid`
- `trend_threshold_total_shell_loc_invalid`

These markers are asserted in `scripts/ci/test_check_kolme_wrapper_budget_trend.sh`.

## Ignored-Test Governance Policy

Ignored-test governance uses deterministic fixture + metadata policy contracts:

- Baseline fixture: `fixtures/ci/ignored_test_inventory_baseline.json`
- Metadata fixture: `fixtures/ci/ignored_test_inventory_metadata.json`
- Promotion criteria fixture: `fixtures/ci/ignored_test_promotion_criteria.json`
- Checker entrypoint: `scripts/ci/check_ignored_test_inventory_drift.sh`

Required metadata fields per ignored test:

- `owner`: non-empty string
- `reason`: non-empty string mapped to a promotion-criteria category
- `priority`: one of `P0`, `P1`, `P2`, `P3`
- `tracking_issue`: required as `#<id>` when `priority` is `P0` or `P1`

Parser and fixture format contract coverage:

- `scripts/ci/test_ignored_test_inventory_parser_contract.sh`

Fast-gate and selector enforcement:

- `scripts/ci/test_ci_tools.sh` runs ignored-test governance checks in fast and full modes.
- `scripts/ci/select_targets.sh` routes ignored-test fixture/checker changes through low-cost `ci-doc-contract` scope.

Fail-closed reason-code mapping for stale-ignore/rationale drift includes:

- `unexpected_ignored_tests_present`
- `baseline_ignored_tests_missing`
- `ignored_test_metadata_missing`
- `ignored_test_metadata_stale_entry`
- `high_priority_tracking_issue_missing`
- `ignored_test_promotion_criteria_missing`
- `ignored_test_rationale_stale`
