# Plan: Issue #4342

Status: Completed
Issue: #4342

## Approach

1. Add RED assertions in wave wrapper-family trend tests for missing taxonomy/runtime-budget markers.
2. Extend `scripts/ci/kolme_wrapper_inventory_baseline.py` with:
   - deterministic reason taxonomy markers,
   - explicit `reason_codes_value`/`policy_decision` markers,
   - `--max-runtime-seconds` fail-closed runtime budget enforcement.
3. Thread runtime budget through non-Kolme wave-19 wrapper checker.
4. Update CI docs for taxonomy and budget marker contracts.
5. Verify target CI trend tests and clippy/fmt gates.

## Affected Modules

- `scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/kolme_wrapper_inventory_baseline.py`
- `scripts/ci/check_non_kolme_wave19_wrapper_family_budget_trend.sh`
- `scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: marker additions break existing parser expectations.
  - Mitigation: additive marker fields; retain existing `reason_codes` marker.
- Risk: runtime budget defaults too strict and cause flaky failures.
  - Mitigation: conservative default budget + explicit override flag/env marker.

## Interfaces / Contracts

- Reason taxonomy marker:
  - `reason_taxonomy_version=kamn.ci.wrapper-budget-trend-reason-taxonomy.v1`
- Runtime budget markers:
  - `ci_smoke_budget_status=within|exceeded`
  - `ci_smoke_max_runtime_seconds=<float>`
  - `ci_smoke_elapsed_seconds=<float>`

## ADR

No ADR required.
