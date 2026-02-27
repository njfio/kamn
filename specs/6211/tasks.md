# Tasks: Issue 6211 - Replace Censorship Ratio f64 Arithmetic with Integer Math

- Issue: #6211
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regression tests for fractional floor behavior and large-value safety.
- [x] T2 (GREEN): replace float ratio path with integer arithmetic helper in watchdog.
- [x] T3 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-runtime-guards -- -D warnings`, and `cargo test -p kamn-runtime-guards`.
