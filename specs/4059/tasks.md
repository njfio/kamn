# Issue #4059 Tasks

- Issue: #4059
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Ordered Tasks
- [x] T1 (RED): add Rust contract tests for audit-integrity GO path, policy pass path, tamper fail-closed path, and bounded dry-run runtime (`C-01`..`C-03`, `C-05`).
- [x] T2 (GREEN): implement any missing test helpers/fixtures to execute existing deploy generator/policy scripts deterministically (`C-01`, `C-02`).
- [x] T3 (GREEN): update `docs/ci/strategy.md` with audit-integrity dry-run governance command and marker contracts (`C-04`).
- [x] T4 (GREEN): extend `ci_strategy_docs.rs` with docs parity assertions for new audit-integrity strategy markers (`C-04`).
- [x] T5 (Regression): validate tampered audit-integrity bundle payload rejection remains fail-closed with deterministic mismatch phrase (`C-03`).
- [x] T6 (Verify): run targeted fmt/clippy/tests and capture CI-safe performance evidence (`C-05`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | helper assertions for marker extraction and JSON payload validation |
| Functional | bundle generation + checker GO behavior |
| Integration | generate + policy-check composition through deploy scripts |
| Regression | tampered audit-integrity gate payload reject behavior |
| Performance | bounded dry-run execution time assertion |
