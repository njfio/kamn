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
bash scripts/kolme/run_message_proof_anchoring_contract_lane.sh
bash scripts/kolme/validate_message_proof_anchoring_live.sh
```

## Live Validation Protocol

- Contract lane:
  - `scripts/kolme/run_message_proof_anchoring_contract_lane.sh`
  - deterministic markers:
    - `status=pass`
    - `final_decision=GO`
    - `message_anchor_contract_status=verified`
    - `lifecycle_alignment_status=verified`
    - `conflict_fail_closed_status=verified`
    - `performance_budget_status=verified`
- Live validation lane:
  - `scripts/kolme/validate_message_proof_anchoring_live.sh`
  - deterministic markers:
    - `status=pass`
    - `final_decision=GO`
    - `message_anchor_contract_status=verified`
    - `evidence_bundle_status=verified`
    - `docs_contract_status=verified`
    - `fail_closed_status=verified`
    - `fail_closed_reason_code=message_proof_anchor_conflicting_key`
    - `performance_budget_status=verified`
