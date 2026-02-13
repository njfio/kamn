# CI Cost and Lane Framework

## Purpose
Keep merge-critical CI fast and cost-bounded while preventing silent growth in shell-lane maintenance surface.

## Budget Sources
- Runtime/runner budgets: `.ci/ci-budget.env`
- Fast-gate delta baseline/thresholds: `.ci/fast-gate-budget-delta.env`
- Script-surface budgets: `.ci/script-surface-budget.env`
- Script-surface delta baseline: `.ci/script-surface-baseline.env`
- Kolme command-surface soft budget: `.ci/kolme-command-surface-soft-budget.env`
- Kolme command-surface baseline: `.ci/kolme-command-surface-baseline.env`
- Kolme command-surface trend thresholds: `.ci/kolme-command-surface-trend-thresholds.env`

## Enforcers
- Runtime budget gate: `scripts/ci/evaluate_budget.sh`
- Fast-gate delta report generator: `scripts/ci/generate_fast_gate_budget_delta_report.sh`
- Fast-gate delta threshold gate: `scripts/ci/check_fast_gate_budget_delta_threshold.sh`
- Script surface gate: `scripts/ci/check_script_duplication_budget.sh`
- Kolme harness+command-surface trend report: `scripts/ci/generate_kolme_test_harness_loc_trend_report.sh`
- CI helper regression suite: `scripts/ci/test_ci_tools.sh`

## Fast-Gate Delta Policy
`scripts/ci/generate_fast_gate_budget_delta_report.sh` compares current fast-gate telemetry against a versioned baseline and emits:

- baseline runtime/cost (`elapsed_seconds`, `runner_minutes`)
- current runtime/cost
- absolute and percentage variance

`scripts/ci/check_fast_gate_budget_delta_threshold.sh` fails closed when positive variance exceeds configured limits without a valid waiver.

## Script-Surface Budget Policy
`scripts/ci/check_script_duplication_budget.py` computes deterministic metrics over non-test shell command surface under `scripts/**/*.sh`:

- files named `test_*.sh` are excluded from metric totals
- symlink wrappers remain counted for `script_count`/`shell_line_total`
- symlink wrappers are excluded from `duplicate_content`

- `script_count`
- `shell_line_total`
- `duplicate_basename`
- `duplicate_content` (regular files only)

The checker also computes per-PR deltas against `.ci/script-surface-baseline.env` and emits:

- `delta_script_count`
- `delta_shell_line_total`
- `delta_duplicate_basename`
- `delta_duplicate_content`

The checker fails closed when any metric exceeds its configured threshold and emits deterministic remediation guidance.

## Waiver Rules
Temporary exceptions are allowed through `.ci/script-surface-budget-waiver.json`.
The repository currently runs without this waiver file; budget checks stay fail-closed with no waiver present.
Required fields:

- `reason` (non-empty string)
- `expires_on` (`YYYY-MM-DD`)
- `allow_metrics` (non-empty string list)

Policy constraints:

- Expired waivers fail closed.
- Malformed waivers fail closed.
- Only explicitly listed metrics are waived.

Fast-gate delta overruns may be waived through `.ci/fast-gate-budget-delta-waiver.json`.
Required fields:

- `reason` (non-empty string)
- `expires_on` (`YYYY-MM-DD`)
- `allow_metrics` (non-empty string list; allowed values: `elapsed_seconds_delta_pct`, `runner_minutes_delta_pct`)

Escalation path for temporary overruns:

1. Open/update a tracking issue with root cause and expected rollback date.
2. Add waiver with explicit metric scope and short expiry.
3. Remove waiver in the follow-up PR after regression is resolved.

## Local Validation
Run these before opening a PR that modifies CI/lane surfaces:

```bash
bash scripts/ci/check_script_duplication_budget.sh
bash scripts/ci/test_check_script_duplication_budget.sh
bash scripts/ci/test_generate_fast_gate_budget_delta_report.sh
bash scripts/ci/test_check_fast_gate_budget_delta_threshold.sh
bash scripts/ci/test_ci_tools.sh
```

Kolme harness + command-surface trend validation:

```bash
bash scripts/ci/generate_kolme_test_harness_loc_report.sh --output-json /tmp/kolme-test-harness-loc-report.json
bash scripts/ci/check_kolme_test_harness_loc_soft_budget.sh --report-file /tmp/kolme-test-harness-loc-report.json --output-json /tmp/kolme-test-harness-loc-soft-budget-report.json
bash scripts/ci/generate_kolme_test_harness_loc_trend_report.sh --output-json /tmp/kolme-test-harness-loc-trend-report.json
```

Deterministic command-surface drift markers in trend artifacts:
- `command_surface_trend_status=within|warn|fail|invalid`
- `command_surface_policy_decision=GO|WARN|NO-GO`
- `command_surface_script_count_trend_warn_delta_exceeded`
- `command_surface_script_count_trend_fail_delta_exceeded`
- `command_surface_shell_line_total_trend_warn_delta_exceeded`
- `command_surface_shell_line_total_trend_fail_delta_exceeded`
- `command_surface_budget_status_fail`

Optional hard-fail mode for command-surface drift:
- `bash scripts/ci/generate_kolme_test_harness_loc_trend_report.sh --enforce-command-surface-fail --output-json /tmp/kolme-test-harness-loc-trend-report.json`

## Artifact Contract
`check_script_duplication_budget.py` supports `--output-json` and writes:

- `schema_version=kamn.ci.script-surface-budget-report.v1`
- metric values
- baseline metric values
- metric deltas
- threshold values
- violation/waiver state
- remediation guidance

This keeps cost governance machine-verifiable for CI and audits.
