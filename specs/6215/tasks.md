# Tasks: Issue 6215 - Clarify Payload Hash Value Semantics in Runtime Commit Identity

- Issue: #6215
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regression proving idempotency key differs for equal-length distinct payload hash values.
- [x] T2 (GREEN): add explicit value-based semantics comments in runtime identity helpers.
- [x] T3 (REGRESSION): run `cargo test -p kamn-kolme`.
- [x] T4 (VERIFY): run `cargo fmt --check` and `cargo clippy -p kamn-kolme -- -D warnings`.

