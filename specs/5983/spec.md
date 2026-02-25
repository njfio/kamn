# Spec: Issue #5983 - Story: Real cross-node message delivery (send->persist->relay->read)

- Issue: #5983
- Status: Reviewed (agent-authored P1; explicit proceed directive on 2026-02-25)
- Type: story
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
`POST /v1/messages/send` currently persists a local message and enqueues a relay spool entry, but daemon relay processing only projects local state (`created -> relayed`) and does not deliver to a recipient node. The product-critical use case (cross-node agent messaging) is not operational end-to-end.

## Scope
In scope:
- Add deterministic recipient routing config for daemon relay forwarding.
- Forward drained relay spool entries from sender node to recipient node over Service API HTTP.
- Persist inbound relayed messages on recipient node and surface them via existing mailbox/query paths.
- Keep relay semantics idempotent and fail closed (no silent drop on forward failure).
- Add integration coverage for two-node send -> daemon relay -> recipient read with restart durability.

Out of scope:
- New p2p/gossip transport protocol.
- Envelope/schema redesign for message payloads.
- Cryptographic algorithm replacement (handled by separate issues).

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Given two service-api nodes and a recipient route mapping, sender `POST /v1/messages/send` plus daemon relay tick forwards to recipient node and persists a retrievable message.
- AC-2: Recipient mailbox/query surfaces relayed message and recipient retrieval transitions `relayed -> delivered` only for the matching recipient DID.
- AC-3: Recipient state remains durable across restart; post-restart query returns the same delivered state.
- AC-4: Relay forward failures do not silently drop messages; failed entries are retained/recoverable for retry and surfaced by deterministic errors.
- AC-5: Relay processing is idempotent for duplicate spool entries and repeated daemon ticks (no duplicate recipient message insertion).

## Conformance Cases
- C-01 (Integration, AC-1): Two-node integration test verifies send on node A, daemon tick forward, and message visibility on node B.
- C-02 (Integration, AC-2): Recipient-only delivery transition test verifies `relayed -> delivered` gating on requester DID.
- C-03 (Regression, AC-3): Restart recipient node and verify persisted message/delivery status remains correct.
- C-04 (Functional/Regression, AC-4): Simulated recipient-unreachable route preserves relay entry for retry and returns deterministic relay error markers.
- C-05 (Regression, AC-5): Duplicate spool entry and repeated tick execution do not duplicate recipient storage records.

## Success Metrics / Observable Signals
- New integration tests pass for two-node delivery and restart durability.
- Existing relay projection tests remain green.
- No regression in service-api auth/rate-limit/concurrency guards.
