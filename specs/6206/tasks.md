# Tasks: Issue 6206 - Unify Task Lifecycle Transition Mapping

- Issue: #6206
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regressions for forward/reverse transition mapping consistency and invalid-edge rejection.
- [x] T2 (GREEN): replace duplicated match tables with one canonical transition edge table.
- [x] T3 (GREEN): route `next_state` and `transition_between` through the canonical table.
- [x] T4 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, and `cargo test -p kamn-core -- task_lifecycle`.
