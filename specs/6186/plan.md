# Plan: Issue 6186 - Durable Replay Guard Across Restart

- Issue: #6186
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extend service-api persisted snapshot schema with sender nonce high-watermark map.
2. Add message-store methods to:
   - read nonce high-watermarks,
   - record/update sender high-watermark with persistence.
3. Extend replay guard with seeded sender nonce floor map and monotonic checks.
4. Seed replay guard from message-store state during endpoint startup.
5. Persist accepted sender nonce in auth middleware after successful authorization.
6. Update replay guard tests to validate monotonic behavior and restart durability.

## Affected Modules

- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs` (test expectations)

## Risks and Mitigations

1. Additional write load per authenticated request:
   - Mitigation: persist only when nonce exceeds stored high-watermark.
2. Behavior change from TTL-only uniqueness to monotonic rejection:
   - Mitigation: explicit regression tests and deterministic reason-code handling.

## Contracts / Interfaces

No external API changes.
Internal auth semantics harden from windowed uniqueness to persisted monotonic replay protection.
