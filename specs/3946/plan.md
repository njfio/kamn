# Issue #3946 Plan

- Issue: #3946
- Status: Completed
- Spec: `specs/3946/spec.md`

## Implementation Approach
1. Add a new `kamn-node` budget contract test that reads a repo fixture and validates shell/fragments line-count thresholds.
2. Run RED before adding fixture (missing fixture failure).
3. Add versioned baseline fixture and docs threshold refresh workflow markers.
4. Run budget/docs regression suites.

## Affected Modules
- `crates/kamn-node/tests/main_tests_surface_budget_contract.rs`
- `fixtures/ci/main_tests_runtime_surface_budget_baseline.json`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: thresholds become stale and cause noisy failures.
  - Mitigation: explicit refresh workflow in docs and fixture version marker.
- Risk: flaky counts from generated files.
  - Mitigation: scope counts to deterministic tracked Rust files in `src/main_tests`.

## Contracts and Interfaces
- Baseline fixture schema: `kamn.node.main-tests-surface-budget-baseline.v1`.
- Reason taxonomy version: `kamn.node.main-tests-surface-budget-reason-taxonomy.v1`.

## Verification Strategy
- RED: budget contract test fails on missing baseline.
- GREEN: add fixture + docs markers and rerun budget suite.
- REGRESSION: run `ci_strategy_docs` to ensure docs contract compatibility.
