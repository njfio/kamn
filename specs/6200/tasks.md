# Tasks: Issue 6200 - Deduplicate Kolme JSON Helper Surface

- Issue: #6200
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add/helper tests for split and whitespace behavior in shared scalar helper module.
- [x] T2 (GREEN): extract shared helper(s) into `json_scalar_policy`.
- [x] T3 (GREEN): remove duplicate helper copies from policy modules and import shared implementations.
- [x] T4 (REGRESSION): run `cargo test -p kamn-kolme` to validate no behavior drift.
- [x] T5 (VERIFY): run `cargo fmt --check` and `cargo clippy -p kamn-kolme -- -D warnings`.

