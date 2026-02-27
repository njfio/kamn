# Tasks: Issue 6186 - Durable Replay Guard Across Restart

- Issue: #6186
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing replay-guard tests for seeded monotonic rejection and stale nonce replay across TTL/capacity eviction (`C-03`).
- [x] T2 (GREEN): add persisted nonce high-watermark storage + replay guard seeding at startup (`C-01`, `C-02`).
- [x] T3 (GREEN): persist accepted auth nonce high-watermark from middleware and keep fail-closed persistence errors (`C-01`).
- [x] T4 (REGRESSION): run service-api auth/replay tests and startup integration paths (`C-02`, `C-03`).
- [x] T5 (VERIFY): run `cargo fmt --check`, scoped clippy, and targeted `kamn-node` tests.

## Test Tier Mapping

- Unit: replay guard monotonic and seeded behavior.
- Integration: persisted nonce high-watermark in state snapshot and startup seeding behavior.
- Regression: existing auth replay reason-code behavior remains deterministic.
