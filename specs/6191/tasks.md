# Tasks: Issue 6191 - Extract Shared Snapshot/Journal Helpers

- Issue: #6191
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): identify shared journal helper duplication points and assert existing corrupt-tail behavior in tests.
- [x] T2 (GREEN): add shared `snapshot_journal` module and switch message/channel/task snapshot stores to it.
- [x] T3 (REGRESSION): run focused `kamn-core --lib` journal regression tests for message/channel/task stores.
- [x] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -p kamn-core -- -D warnings`.
