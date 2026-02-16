# Issue #4321 Tasks

- Issue: `#4321`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red): add persisted block replay mismatch/tamper tests covering required categories.
- T2 (Docs): update release go/no-go checklist mismatch/tamper failure markers and docs test coverage.
- T3 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`

## Completion Evidence
- Persisted replay digest/checkpoint/tamper failure modes are protected by deterministic tests.
- Release checklist mismatch/tamper markers are enforced through docs tests.
