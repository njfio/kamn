# Issue #4017 Tasks

- Issue: #4017
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Ordered Tasks

- [x] T1 (RED): add failing crash-restart lane runner contract tests + ops docs marker assertion (`C-01`..`C-07`).
- [x] T2 (GREEN): implement profile-aware local-heavy runner wrapper and deterministic artifact schema output (`C-01`..`C-06`).
- [x] T3 (GREEN): update ops docs crash-restart artifact marker table (`C-07`).
- [x] T4 (VERIFY): run targeted tests, fmt, and clippy; finalize status updates (`C-01`..`C-07`).

## Test Tier Mapping

| Tier | Planned Coverage |
|---|---|
| Unit | dry-run schema/taxonomy marker contract |
| Functional | restart/corruption profile behavior |
| Integration | combined profile artifact projection + source parity |
| Regression | invalid-profile fail-closed + docs marker drift checks |
| Performance | bounded dry-run runtime budget |

## Dependency Notes

- T2 depends on T1 RED evidence.
- T3 depends on finalized marker names from T2.
- T4 depends on T2/T3 completion.
