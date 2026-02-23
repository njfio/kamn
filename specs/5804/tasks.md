# Tasks: Issue #5804 - Add kamn-core Live Probe Matrix Module

- Issue: #5804
- Spec: `specs/5804/spec.md`
- Plan: `specs/5804/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (Conformance/RED): add failing contract tests for matrix validation/aggregation behavior.
- [x] T2 (GREEN): implement `live_probe_matrix` module and export through `lib.rs`.
- [x] T3 (Regression): run targeted kamn-core tests + fmt checks.
- [x] T4 (Regression): preserve spec-volume cap while adding `specs/5804`; run R50/R53 docs-contract non-regression suites.
- [x] T5 (Closeout): finalize artifact statuses and milestone metadata.

## Tier Mapping
- Unit: module-local tests for validation and helper behavior.
- Functional: contract test for mixed mode/scenario matrices.
- Conformance: public export and lifecycle/milestone artifact closure.
- Regression: docs-contract non-regression suites (R50/R53).
- Performance: N/A (no performance-sensitive logic introduced).
