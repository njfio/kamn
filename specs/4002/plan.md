# Issue #4002 Plan

## Approach
1. Extend `scripts/ci/performance_smoke_contracts.py` check mode with deterministic reason-taxonomy outputs and fail-closed reason normalization.
2. Add selector/workflow contract checks in checker inputs to enforce smoke-only fast-gate path invariants.
3. Add Rust governance contract suite (`performance_ci_smoke_governance_contract.rs`) covering unit/functional/integration/regression/performance categories.
4. Update `scripts/ci/test_ci_tools.sh` fast-mode coverage to include the Rust governance test.
5. Update `docs/ci/strategy.md` with performance checker threshold and exclusion contract markers.

## Affected Modules
- `scripts/ci/performance_smoke_contracts.py`
- `scripts/ci/check_performance_thresholds.sh` (wrapper contract preserved)
- `scripts/ci/test_check_performance_thresholds.sh`
- `scripts/ci/test_ci_tools.sh`
- `crates/kamn-core/tests/performance_ci_smoke_governance_contract.rs` (new)
- `docs/ci/strategy.md`
- `specs/4002/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: output format changes could break existing shell tests.
  - Mitigation: preserve existing checker command path and update shell test contract in the same change.
- Risk: selector parsing could drift if fast-mode block shape changes.
  - Mitigation: use deterministic fast-mode block extraction and regression test with deliberate drift fixtures.
- Risk: CI runtime overhead increase.
  - Mitigation: enforce runtime budget assertion in performance test and keep checker/file scans bounded.

## Interfaces and Contracts
- Reason taxonomy marker:
  - `performance_ci_smoke_reason_taxonomy_version=kamn.ci.performance-ci-smoke-threshold-reason-taxonomy.v1`
- Reason code catalog marker:
  - `performance_ci_smoke_reason_codes_csv=<deterministic ordered csv>`
- Final decision markers:
  - `status=pass|fail`
  - `final_decision=GO|NO-GO`
  - `reason_codes_value=none|<csv>`
