# Issue #4018 Tasks

- Issue: #4018
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Ordered Tasks

- [x] T1 (RED): add failing checker contract tests + docs parity assertions (`C-01`..`C-06`).
- [x] T2 (GREEN): implement crash-restart policy checker + wrapper + registry wiring (`C-01`..`C-03`).
- [x] T3 (GREEN): add shell checker regression script + ci-tools command-surface wiring (`C-02`..`C-05`).
- [x] T4 (GREEN): update strategy/runbook marker sections + docs tests (`C-03`..`C-05`).
- [x] T5 (VERIFY): run targeted tests + fmt + clippy; finalize issue/PR status updates (`C-01`..`C-06`).

## Test Tier Mapping

| Tier | Planned Coverage |
|---|---|
| Unit | checker accepts valid baseline report and emits deterministic markers |
| Functional | checker rejects tampered corruption/recovery marker payload |
| Integration | runner + checker + strategy/runbook parity composition |
| Regression | runbook or strategy marker drift fails closed with deterministic reason |
| Performance | checker remains within bounded local runtime budget |

## Dependency Notes

- T2 depends on T1 RED evidence.
- T3/T4 depend on finalized checker marker constants from T2.
- T5 depends on T2-T4 completion.
