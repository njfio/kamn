# Tasks: Issue 6213 - CLI Unknown Flags Must Fail Closed

- Issue: #6213
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add parser tests for unknown `--` and `-` flags to fail closed.
- [x] T2 (GREEN): update parser token handling to reject unknown flag tokens deterministically.
- [x] T3 (GREEN): preserve positional passthrough handling for non-flag arguments.
- [x] T4 (REGRESSION): run `cargo test -p kamn-cli`.
- [x] T5 (VERIFY): run `cargo fmt --check` and `cargo clippy -p kamn-cli -- -D warnings`.

