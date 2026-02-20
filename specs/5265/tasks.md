# Issue #5265 Tasks

- Issue: #5265
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing scheduler and adapter persistence transition tests for M1 batch lifecycle contracts.
- [x] T2 (Implementation/GREEN): add deterministic M1 scheduler trigger policy module and public exports.
- [x] T3 (Implementation/GREEN): add adapter methods for merkle batch create/assign/submitted/confirmed state persistence.
- [x] T4 (Regression): add fail-closed invalid payload/state transition tests.
- [x] T5 (Verification): run fmt, strict clippy, and targeted test suites.
- [x] T6 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | scheduler policy validation and reason projection |
| Functional | deterministic trigger evaluation across threshold boundaries |
| Integration | live PostgreSQL merkle batch lifecycle persistence path |
| Regression | invalid identifiers and transition payload failure matrix |
| Performance | N/A (operational contract + persistence wiring slice) |
