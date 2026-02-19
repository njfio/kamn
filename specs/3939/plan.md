# Issue #3939 Plan

- Issue: #3939
- Status: Completed
- Spec: `specs/3939/spec.md`

## Implementation Approach
1. Complete `#3946`: baseline-backed runtime test-surface budget checker.
2. Complete `#3947`: ownership/docs marker contracts.
3. Validate with targeted docs and governance suites.

## Affected Modules
- `crates/kamn-node/tests/main_tests_surface_budget_contract.rs`
- `fixtures/ci/main_tests_runtime_surface_budget_baseline.json`
- `crates/kamn-core/tests/node_test_surface_ownership_docs.rs`
- `docs/ci/strategy.md`
- `docs/foundation/runtime-watchdog-attestation.md`

## Verification Strategy
- RED/GREEN/REGRESSION evidence in merged PRs #5163 and #5164.
