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
