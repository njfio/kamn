# CI Cost and Lane Framework

## Purpose
Keep merge-critical CI fast and cost-bounded while preventing silent growth in shell-lane maintenance surface.

## Budget Sources
- Runtime/runner budgets: `.ci/ci-budget.env`
- Script-surface budgets: `.ci/script-surface-budget.env`

## Enforcers
- Runtime budget gate: `scripts/ci/evaluate_budget.sh`
- Script surface gate: `scripts/ci/check_script_duplication_budget.sh`
- CI helper regression suite: `scripts/ci/test_ci_tools.sh`

## Script-Surface Budget Policy
`scripts/ci/check_script_duplication_budget.py` computes deterministic metrics over `scripts/**/*.sh`:

- `script_count`
- `shell_line_total`
- `duplicate_basename`
- `duplicate_content`

The checker fails closed when any metric exceeds its configured threshold.

## Waiver Rules
Temporary exceptions are allowed through `.ci/script-surface-budget-waiver.json`.
Required fields:

- `reason` (non-empty string)
- `expires_on` (`YYYY-MM-DD`)
- `allow_metrics` (non-empty string list)

Policy constraints:

- Expired waivers fail closed.
- Malformed waivers fail closed.
- Only explicitly listed metrics are waived.

## Local Validation
Run these before opening a PR that modifies CI/lane surfaces:

```bash
bash scripts/ci/check_script_duplication_budget.sh
bash scripts/ci/test_check_script_duplication_budget.sh
bash scripts/ci/test_ci_tools.sh
```

## Artifact Contract
`check_script_duplication_budget.py` supports `--output-json` and writes:

- `schema_version=kamn.ci.script-surface-budget-report.v1`
- metric values
- threshold values
- violation/waiver state

This keeps cost governance machine-verifiable for CI and audits.
