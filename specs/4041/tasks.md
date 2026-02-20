# Issue #4041 Tasks

- Issue: #4041
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Ordered Tasks
- [x] T1 (RED): add failing Rust contract tests for lane run/policy/contract behavior and docs parity markers (`C-01`..`C-06`).
- [x] T2 (GREEN): implement version-policy fixture parser + run-lane deterministic report projection (`C-01`, `C-02`).
- [x] T3 (GREEN): implement fail-closed checker and contract-lane tamper/docs parity checks (`C-03`, `C-04`, `C-05`).
- [x] T4 (GREEN): wire `exec_dispatch` wrappers and registry entries without new shell script bodies.
- [x] T5 (GREEN): update strategy/ops docs and docs-contract test assertions for marker parity (`C-05`).
- [x] T6 (VERIFY): run targeted fmt/clippy/tests and capture performance/runtime evidence (`C-06`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | fixture parser + dry-run report schema projection |
| Functional | supported/unsupported window evaluation + policy pass/fail |
| Integration | contract lane composition with docs parity checks |
| Regression | tampered marker fail-closed reason contracts |
| Performance | dry-run contract lane bounded runtime assertion |
