# Issue #4044 Tasks

- Issue: #4044
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Ordered Tasks
- [x] T1 (RED): add Rust checker tests for pass path, tampered-report failure, selector/workflow exclusion drift, docs parity drift, and runtime budget contract (`C-01`..`C-06`).
- [x] T2 (GREEN): add baseline threshold fixture under `fixtures/ci/` and implement parser validation (`C-03`).
- [x] T3 (GREEN): implement CI dry-run compatibility governance checker in `scripts/ci/` (`C-01`, `C-02`, `C-04`, `C-05`, `C-06`).
- [x] T4 (GREEN): update strategy/ops docs with deterministic checker, threshold, reason taxonomy, and remediation markers (`C-05`).
- [x] T5 (Integration): wire checker contract test into `scripts/ci/test_ci_tools.sh` fast/full paths (`C-04`).
- [x] T6 (VERIFY): run targeted fmt/clippy/tests and shell-surface guardrail checks (`C-06`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | threshold parsing + valid report contract evaluation |
| Functional | checker pass/fail behavior for compatibility report drift |
| Integration | selector/workflow exclusion and docs parity composition |
| Regression | tampered report + remediation marker drift fail-closed reasons |
| Performance | checker runtime bounded by threshold |
