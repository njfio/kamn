# Tasks: Issue 6210 - Bound Anti-Spam Seen-Message-ID Growth

- Issue: #6210
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regression tests that prove deterministic eviction of oldest seen message IDs at configured capacity.
- [x] T2 (RED): add regression test that rejects `max_seen_message_ids=0` at engine construction.
- [x] T3 (GREEN): add bounded seen-id capacity to `AntiSpamConfig` with default and validation.
- [x] T4 (GREEN): add FIFO seen-id retention/eviction path in `AntiSpamEngine::evaluate`.
- [x] T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-runtime-guards -p kamn-node -- -D warnings`, and `cargo test -p kamn-runtime-guards`.
