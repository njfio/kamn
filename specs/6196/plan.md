# Plan: Issue 6196 - Monotonic Nonce Contract Across TTL Windows

- Issue: #6196
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Keep replay guard keyed by sender+nonce for short-window duplicate detection.
2. Enforce sender high-watermark monotonicity before duplicate-set acceptance checks.
3. Persist and hydrate sender high-watermarks through message-store state.
4. Add explicit post-TTL replay regression coverage.

## Affected Modules

- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`

## Risks and Mitigations

1. Risk: accidental nonce acceptance after guard-entry TTL eviction.
   - Mitigation: high-watermark check is independent of entry-set/TTL and tested directly.
2. Risk: restart could reset monotonic floor.
   - Mitigation: persisted high-watermark snapshot state + startup seeding test coverage.
