# Issue #4015 Tasks

- Issue: #4015
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Ordered Tasks

- [x] T1 (RED): add failing contract tests for fixture parser, journal schema, commit-boundary markers, and docs marker section (`C-01`..`C-06`).
- [x] T2 (GREEN): add fixture file `fixtures/runtime/journal_wal_commit_boundary_fixture_matrix.txt` (`C-01`, `C-03`..`C-06`).
- [x] T3 (GREEN): update `docs/foundation/runtime-network.md` with journal/WAL marker map (`C-04`, `C-06`).
- [x] T4 (VERIFY): run targeted tests, `fmt`, and scoped `clippy`; update task statuses (`C-01`..`C-06`).

## Test Tier Mapping

| Tier | Planned Coverage |
|---|---|
| Unit | fixture parser schema/columns validation |
| Functional | journal record schema validation for valid writes |
| Integration | store recovery reason-code parity for empty/clean/repaired/corrupt-tail paths |
| Regression | fixture/docs marker drift fail-closed checks (`Regression: #4015`) |
| Performance | N/A |

## Dependency Notes

- T2 depends on T1 RED evidence.
- T3 depends on T2 marker names.
- T4 depends on T2/T3 completion.
