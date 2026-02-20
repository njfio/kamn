# Issue #4016 Tasks

- Issue: #4016
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Ordered Tasks

- [x] T1 (RED): add failing partial-write matrix tests and ops-doc marker assertions (`C-01`..`C-06`).
- [x] T2 (GREEN): adjust file-store read precedence to honor valid journal commits before snapshot-file fallback (`C-01`, `C-02`, `C-03`).
- [x] T3 (GREEN): add fixture `fixtures/runtime/journal_wal_partial_write_fault_matrix.txt` and docs markers in `docs/ops/configuration.md` (`C-04`, `C-05`).
- [x] T4 (VERIFY): run targeted tests + fmt/clippy and finalize status updates (`C-01`..`C-06`).

## Test Tier Mapping

| Tier | Planned Coverage |
|---|---|
| Unit | fixture parser schema/columns validation |
| Functional | partial snapshot-file write recovery to latest journal commit |
| Integration | cross-store fault-mode matrix (partial snapshot, partial journal tail, no-journal repair) |
| Regression | deterministic taxonomy + docs parity drift fail-closed checks (`Regression: #4016`) |
| Performance | bounded fault-matrix runtime budget assertion |

## Dependency Notes

- T2 depends on T1 RED evidence.
- T3 depends on final marker naming from T2.
- T4 depends on T2/T3 completion.
