# Spec: Issue #6060 - Durable replay guard nonce floors for Service API auth

- Issue: #6060
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6059

## Problem Statement
Service API replay protection currently keeps recent `(sender_did, nonce)` entries only in memory. After process restart, the replay set is empty and previously accepted nonces can be reused. This creates a restart-window replay risk and keeps authentication integrity coupled to uptime.

## Scope
In scope:
- Persist per-sender nonce floor for replay guard state.
- Reload persisted nonce floors when service API starts.
- Enforce strict monotonic nonce progression per sender (`nonce > last_seen_nonce`).
- Preserve bounded in-memory replay-window behavior (TTL/capacity) for short-horizon duplicate detection.

Out of scope:
- DID registry redesign.
- Multi-key/per-agent trust model redesign.
- Transport protocol header changes.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Replay guard stores per-sender nonce floors durably and restores them after restart.
- AC-2: Auth rejects nonce values less than or equal to the stored sender nonce floor.
- AC-3: Existing replay-window contracts (capacity + TTL behavior) remain passing.
- AC-4: Startup with no replay state file succeeds and initializes empty persistent replay state.

## Conformance Cases
- C-01 (Integration, AC-1): Record a sender nonce, reconstruct replay guard from same state path, and assert old nonce is rejected while higher nonce is accepted.
- C-02 (Functional, AC-2): For one sender, lower/equal nonce attempts are rejected once a higher nonce is accepted.
- C-03 (Regression, AC-3): Existing replay guard capacity and TTL regression tests continue to pass unchanged.
- C-04 (Unit, AC-4): Replay guard construction with absent state path yields empty nonce floor and accepts first nonce.

## Success Metrics / Observable Signals
- RED test demonstrates restart replay acceptance before fix.
- GREEN test demonstrates restart replay rejection after fix.
- `cargo test -p kamn-node replay_guard` passes with new and existing replay tests.
