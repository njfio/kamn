# Issue #3946 Tasks

- Issue: #3946
- Status: Completed

## Ordered Tasks
- [x] T1 (Red): add budget contract test and capture failing run with missing fixture.
- [x] T2 (Green): add baseline fixture and threshold parsing/check logic.
- [x] T3 (Green): document threshold markers and refresh workflow in `docs/ci/strategy.md`.
- [x] T4 (Regression): run budget contract and docs-contract suites.
- [x] T5 (Verify): record issue closure evidence and DoD markers.

## Tier Mapping
- Unit: fixture/schema parser validation in budget contract test.
- Functional: shell/fragment threshold checks.
- Integration: docs + fixture + contract checker composition.
- Regression: fail-closed drift assertions for threshold/baseline changes.
- Performance: N/A (test/docs governance scope).
