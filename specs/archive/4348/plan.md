# Plan: Issue #4348

Status: Completed
Issue: #4348

## Approach

1. Add constants and marker fields to `kolme_wrapper_inventory_baseline.py` for reason taxonomy.
2. Add `--max-runtime-seconds` support and fail-closed budget check.
3. Emit runtime budget status/elapsed/max markers in stdout and report payload.
4. Pass runtime budget defaults through wave-19 wrapper script.
5. Update docs and verify CI contract suites.

## Affected Modules

- `scripts/ci/kolme_wrapper_inventory_baseline.py`
- `scripts/ci/check_non_kolme_wave19_wrapper_family_budget_trend.sh`
- `scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: existing tools consume only older marker set.
  - Mitigation: keep old markers intact and add new marker fields additively.

## Interfaces / Contracts

- New reason code:
  - `ci_smoke_runtime_budget_exceeded`

## ADR

No ADR required.
