# Plan: Issue #4347

Status: Completed
Issue: #4347

## Approach

1. Extend shared non-Kolme wave wrapper trend test harness assertions for taxonomy marker lines.
2. Add deterministic over-budget scenario (`--max-runtime-seconds 0`) expecting fail-closed reason mapping.
3. Capture RED failures before implementation.

## Affected Modules

- `scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`

## Risks / Mitigations

- Risk: runtime over-budget scenario may not trigger deterministically.
  - Mitigation: enforce strict zero-second budget and high-resolution runtime comparison in implementation.

## Interfaces / Contracts

- Expected markers:
  - `reason_taxonomy_version=kamn.ci.wrapper-budget-trend-reason-taxonomy.v1`
  - `reason_codes_value=none|<csv>`
  - `ci_smoke_budget_status=within|exceeded`

## ADR

No ADR required.
