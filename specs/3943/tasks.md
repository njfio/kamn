# Issue #3943 Tasks

- Issue: #3943
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add panic-policy docs-contract assertions that fail before remediation markers are added.
- [x] T2 (Green): add deterministic remediation markers in `docs/ci/strategy.md`.
- [x] T3 (Regression): run targeted and full `ci_strategy_docs` test suite.
- [x] T4 (Verify): ensure docs-contract parity closure is reflected in issue and spec artifacts.

## Tier Mapping
- Unit: panic-policy marker assertions in docs-contract test.
- Functional: docs section contains checker/taxonomy/remediation markers.
- Integration: full `ci_strategy_docs` suite includes panic-policy marker parity test.
- Regression: fail-closed guard for missing remediation markers.
- Performance: N/A (docs/test-only updates).
