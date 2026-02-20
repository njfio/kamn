# Issue #4014 Tasks

- Issue: #4014
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Ordered Tasks
- [ ] T1 (RED): add Rust checker tests for pass path, tampered-report failure, selector/workflow exclusion drift, docs remediation parity drift, and runtime budget contract (`C-01`..`C-06`).
- [ ] T2 (GREEN): add threshold fixture under `fixtures/ci/` and parser validation (`C-03`).
- [ ] T3 (GREEN): implement durability CI dry-run checker in `scripts/ci/` (`C-01`, `C-02`, `C-04`, `C-05`, `C-06`).
- [ ] T4 (GREEN): update strategy/ops docs with checker, threshold, reason taxonomy, and remediation markers (`C-05`).
- [ ] T5 (Integration): wire checker contract test into `scripts/ci/test_ci_tools.sh` fast/full paths (`C-04`).
- [ ] T6 (VERIFY): run targeted fmt/clippy/tests and shell-surface guardrails (`C-06`).

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | threshold parsing + valid report contract evaluation |
| Functional | checker fail-closed on report contract drift |
| Integration | selector/workflow exclusion and docs parity composition |
| Regression | docs remediation marker drift and deterministic reason mapping |
| Performance | checker runtime bounded by threshold |
