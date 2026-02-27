# Spec: Issue 6196 - Monotonic Nonce Contract Across TTL Windows

- Issue: #6196
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: security

## Problem Statement

Replay guard entries have TTL-based eviction. Without monotonic sender nonce enforcement,
requests with stale nonces could be replayed after TTL expiration.

## Scope

In scope:
1. Enforce per-sender monotonic nonce acceptance independent of TTL eviction.
2. Prove stale nonce rejection after TTL in deterministic unit tests.
3. Preserve restart continuity for sender nonce high-watermarks.

Out of scope:
1. Distributed cross-node nonce consensus.
2. Replacing replay guard data structure design.

## Acceptance Criteria

### AC-1 Monotonic Sender Nonce
Given sender nonce history,
When a nonce less than or equal to the sender high-watermark is received,
Then auth fails closed even if replay-set entries are evicted by TTL.

### AC-2 Post-TTL Replay Rejection
Given a sender nonce accepted before TTL expiry,
When the same or lower nonce is replayed after TTL,
Then replay guard rejects the request.

### AC-3 Restart Continuity
Given persisted sender nonce high-watermark state,
When the service restarts,
Then stale nonce values remain rejected.

## Conformance Cases

- C-01 (AC-1, Unit): `service_api_endpoint::auth::tests::regression_issue_6196_nonce_contract_rejects_post_ttl_replay_nonce_values`
- C-02 (AC-2, Unit): `service_api_endpoint::auth::tests::regression_replay_guard_ttl_eviction_rejects_only_within_active_window`
- C-03 (AC-3, Integration): `service_api_endpoint::tests::integration_message_store_persists_auth_nonce_high_watermark_state`
