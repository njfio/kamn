# Plan: Issue #4453

Status: Completed
Issue: #4453

## Approach

1. Add RED assertions to script tests for deterministic taxonomy markers on the generic
   soft-budget checker and for bounded CI-smoke markers on the generic contract lane.
2. Implement taxonomy constants/normalization in
   `scripts/ci/check_test_harness_loc_soft_budget.py`.
3. Implement deterministic CI-smoke boundary markers in
   `scripts/ci/test_harness_loc_soft_budget_contract_lane_impl.sh` output/report payload.
4. Update `docs/ci/strategy.md` with structural-budget reason taxonomy and CI-smoke
   enforcement marker references.
5. Add docs contract assertions in `crates/kamn-core/tests/ci_strategy_docs.rs`.
6. Run RED/GREEN loops and scoped verification.

## Affected Modules

- `scripts/ci/check_test_harness_loc_soft_budget.py`
- `scripts/ci/test_check_test_harness_loc_soft_budget.sh`
- `scripts/ci/test_harness_loc_soft_budget_contract_lane_impl.sh`
- `scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4453/*`

## Risks and Mitigations

- Risk: downstream parsing could depend on existing output-only fields.
  - Mitigation: preserve existing markers; only add new deterministic markers.
- Risk: docs contract file is large and brittle.
  - Mitigation: add a focused test function with stable marker strings for this issue.

## Interfaces / Contracts

- Soft-budget checker output contract adds:
  - `reason_taxonomy_version`
  - `reason_codes_csv`
  - `reason_codes_value`
  - `reason_class`
- Contract-lane output/report contract adds:
  - `ci_smoke_lane_cost_profile`
  - `ci_smoke_runtime_budget_status`
  - existing `reason_key` remains deterministic and documented.

## ADR

Not required: no dependency or architecture change; this is deterministic output
normalization and CI lane contract hardening.
