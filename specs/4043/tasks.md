# Issue #4043 Tasks

- Issue: #4043
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Ordered Tasks
- [x] T1 (RED): add failing Rust contract tests for local-heavy matrix lane/policy/contract behavior and ops marker parity (`C-01`..`C-06`).
- [x] T2 (GREEN): implement fixture parser + deterministic matrix artifact schema projection (`C-01`, `C-02`).
- [x] T3 (GREEN): implement fail-closed checker and contract-lane tamper rejection (`C-03`, `C-04`, `C-05`).
- [x] T4 (GREEN): wire `exec_dispatch` wrappers and registry entries with zero new shell script bodies.
- [x] T5 (GREEN): update `docs/ops/configuration.md` markers and docs-contract assertions (`C-05`).
- [x] T6 (VERIFY): run targeted fmt/clippy/tests and performance evidence (`C-06`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | dry-run artifact schema and deterministic markers |
| Functional | compatibility class projection + policy pass/fail |
| Integration | contract lane composition for lane + policy |
| Regression | tampered matrix marker fail-closed reason |
| Performance | dry-run contract-lane runtime bounded |
