# Spec: Issue 6186 - Durable Replay Guard Across Restart

- Issue: #6186
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: security

## Problem Statement

Replay guard nonce tracking is in-memory only. After restart, previous nonce history is lost,
allowing replay of previously accepted requests.

## Scope

In scope:
1. Persist replay nonce high-watermark state to service-api state storage.
2. Seed replay guard from persisted state during startup.
3. Enforce monotonic nonce checks so old/replayed nonces are rejected across restart.

Out of scope:
1. Cross-node distributed replay coordination.
2. Replay-proof cryptographic nonce commitments.
3. DB-backed anti-spam state unification.

## Acceptance Criteria

### AC-1 Persisted Nonce High-Watermark
Given authenticated request success,
When nonce is accepted,
Then sender nonce high-watermark is persisted in service-api state file.

### AC-2 Restart Durability
Given persisted sender nonce high-watermarks,
When service-api runtime starts,
Then replay guard seeds from persisted values before serving auth-protected requests.

### AC-3 Monotonic Replay Rejection
Given a sender with recorded nonce `N`,
When a request uses nonce `<= N`,
Then request is rejected as replay.

## Conformance Cases

- C-01 (AC-1, Integration): accepted auth request persists sender nonce high-watermark field in state snapshot.
- C-02 (AC-2, Unit/Integration): startup seeding loads persisted high-watermark into replay guard.
- C-03 (AC-3, Unit): replay guard rejects stale nonces after seeding and across TTL/capacity eviction.

## Success Signals

1. Replay guard no longer resets to empty replay history after restart.
2. State snapshot includes deterministic nonce high-watermark map.
3. Replay tests confirm stale nonce rejection after restart simulation.
