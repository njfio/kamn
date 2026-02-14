# Message Anchoring Architecture

This document captures Phase 4.3 core delivery for message proof anchoring on Kolme
(Task #2941, Subtask #2942).

## Core Components

- `MessageProofAnchoringService`
  - lifecycle-aware submission coordinator
  - deterministic idempotency and retry classification
  - finality tracking by message id
- `MessageProofChainAdapter`
  - chain adapter abstraction for proof anchor submission
- `InMemoryMessageProofChainAdapter`
  - deterministic low-cost adapter for CI and contract tests
- `KolmeMessageProofChainAdapter`
  - projects anchor requests into deterministic `KolmeRuntimeCommitRequest`

## Lifecycle Alignment

Entry point:
- `anchor_message_proof_via_chain_adapter(...)`

Rules:

1. Message must already be `Broadcast` or `Included`.
2. On `Submitted`, `Duplicate`, or `FinalizedNoOp` outcomes:
   - if current status is `Broadcast`, transition to `Included`
   - if already `Included`, keep state unchanged
3. On `Rejected`, lifecycle state remains unchanged.

## Retry and Finality Contracts

- Retry classes:
  - `NewSubmission`
  - `RetryableInFlight`
  - `FinalizedNoRetry`
  - `ConflictNoRetry`
- Finality API:
  - `record_anchor_finality(message_id, key, sequence, status, receipt)`
  - `anchor_finality(message_id)`

Deterministic fail-closed errors:
- `InvalidAnchorState`
- `ConflictingAnchorIdempotencyKey`
- `UnknownAnchorIdempotencyKey`
- `ChainAdapterSubmitFailed`

## Validation Commands

```bash
cargo test -p kamn-core --test message_proof_anchoring
cargo test -p kamn-core --test message_proof_anchoring_docs
cargo clippy -p kamn-core -- -D warnings
cargo fmt --check
```
