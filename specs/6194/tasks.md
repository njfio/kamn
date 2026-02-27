# Tasks: Issue 6194 - Message Expiration Must Use Lifecycle Transition Contract

- Issue: #6194
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regression coverage for validated overdue message expiration.
- [x] T2 (GREEN): route expiration APIs through `transition(..., Expired)` and update transition edges.
- [x] T3 (REGRESSION): run focused lifecycle unit tests in `kamn-core --lib`.
- [x] T4 (VERIFY): run `cargo fmt --check` and scoped `cargo clippy -p kamn-core -- -D warnings`.
