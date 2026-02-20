# Issue #5269 Tasks

- Issue: #5269
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing follow-up policy projection tests for retry/poll/no-retry paths.
- [x] T2 (Implementation/GREEN): add orchestrator follow-up policy types and deterministic projection logic.
- [x] T3 (Implementation/GREEN): wire follow-up policy metadata into orchestrator outcomes and exports.
- [x] T4 (Regression): add duplicate-pending retry and rejected/conflict no-retry regression coverage.
- [x] T5 (Verification): run fmt, strict clippy, and targeted tests.
- [x] T6 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | follow-up projection helpers and validation |
| Functional | deterministic retry/poll/no-retry mapping from retry class + finality |
| Integration | persistence-plan + follow-up policy coherence through adapter lifecycle flow |
| Regression | duplicate pending and conflict/rejected deterministic follow-up branches |
| Performance | N/A (policy projection slice only) |
