# Tasks: Issue 6196 - Monotonic Nonce Contract Across TTL Windows

- Issue: #6196
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing regression test for stale nonce replay after TTL expiry.
- [x] T2 (GREEN): enforce sender nonce high-watermark monotonic check in replay guard path.
- [x] T3 (GREEN): preserve sender nonce high-watermark across restart.
- [x] T4 (REGRESSION): run replay/auth-focused node tests and restart continuity tests.
- [x] T5 (VERIFY): run `cargo fmt --check`, scoped clippy, and targeted node test lanes.
