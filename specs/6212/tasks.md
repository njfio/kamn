# Tasks: Issue 6212 - Cache Logging Config Instead of Per-Emission Env Reads

- Issue: #6212
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regressions proving cached success path resolves once and cached error path fails closed.
- [x] T2 (GREEN): implement `OnceLock`-backed cache helper for logging config resolution.
- [x] T3 (GREEN): switch non-test `emit_log_event` to cache-backed config lookup.
- [x] T4 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, and `cargo test -p kamn-node`.
