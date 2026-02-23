# Tasks: Issue #5831 - R55 Gap Closure Contracts and Runtime Surface Reactivation

- Issue: #5831
- Spec: `specs/5831/spec.md`
- Plan: `specs/5831/plan.md`
- Status: Completed

## Ordered Tasks
- [x] T1 (RED/Conformance): add/adjust docs-contract assertions for R55 unresolved closure markers and counting formulas.
- [x] T2 (GREEN/Spec Hygiene): switch remaining spec-dir count helpers to tracked-only semantics and add regression proof against untracked dirs.
- [x] T3 (GREEN/Cap Contract): enforce workspace contract-file counting and R55 cap-status mitigation marker invariants.
- [x] T4 (GREEN/Runtime Surface): implement shared `kamn-kolme` service-auth scope taxonomy and integrate `kamn-node` route-scope enforcement.
- [x] T5 (GREEN/Expect Audit): add deterministic production `expect()` marker validation in R55 docs-contract lanes.
- [x] T6 (Regression): run targeted crate/document lanes and fix regressions.
- [x] T7 (Quality Gates): run fmt, clippy, full workspace tests, then finalize lifecycle statuses.

## Tier Mapping
- Unit: scope parse/render and route-scope mapping checks.
- Functional: node auth scope enforcement behavior remains fail-closed.
- Conformance: review marker arithmetic/counting formulas and unresolved closure markers.
- Integration: workspace route/scope contracts and crate integration lanes.
- Regression: prior docs-contract/spec-volume lanes remain green.
- Performance: N/A (no hotspot algorithm change).
