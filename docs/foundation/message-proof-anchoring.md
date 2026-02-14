# Message Proof Anchoring (Issue #2941)

This document defines deterministic message proof anchoring behavior aligned with
`MessageLifecycleStore` transitions and Kolme runtime-commit submission.

## Behavior

- `MessageProofAnchoringService::anchor_message_proof_via_chain_adapter(...)`:
  - requires lifecycle status to be `Broadcast` or `Included`
  - computes deterministic idempotency key via `idempotency_key_for_anchor(request)`
  - classifies retries as:
    - `NewSubmission`
    - `RetryableInFlight`
    - `FinalizedNoRetry`
    - `ConflictNoRetry`
  - submits through `MessageProofChainAdapter` when retry class is not finalized
  - maps successful submit/duplicate/no-op outcomes to lifecycle alignment:
    - `Broadcast` -> `Included`
    - `Included` remains `Included`
- chain adapter typed outcomes:
  - `Submitted(receipt)`
  - `Duplicate(receipt)`
  - `Rejected { reason }`
  - `FinalizedNoOp`
- `record_anchor_finality(message_id, key, sequence, status, receipt)` enforces:
  - idempotent same-sequence same-payload acceptance
  - stale sequence rejection
  - conflicting sequence rejection
  - unknown key rejection
- deterministic fail-closed errors include:
  - `InvalidAnchorState`
  - `ConflictingAnchorIdempotencyKey`
  - `UnknownAnchorIdempotencyKey`
  - `ChainAdapterSubmitFailed`

## Kolme Integration

- `KolmeMessageProofChainAdapter` maps one anchor request into deterministic
  `KolmeRuntimeCommitRequest` and returns typed submission outcomes.
- `InMemoryMessageProofChainAdapter` provides low-cost deterministic CI coverage.

## Validation Rules

- `message_id` must be non-empty.
- `actor_did` must be a valid KAMN DID.
- `nonce` must be greater than zero.
- `proof_hash` must be non-empty.
- Conflicting same-message idempotency windows fail closed (`Regression: #2941`).

## Local Validation

Run from repository root:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
cargo test -p kamn-core --test message_proof_anchoring
cargo test -p kamn-core --test message_proof_anchoring -- functional_anchor_submission_advances_broadcast_to_included_with_typed_outcome
cargo test -p kamn-core --test message_proof_anchoring -- integration_anchor_retry_is_duplicate_without_reapplying_state_transition
cargo test -p kamn-core --test message_proof_anchoring -- regression_anchor_conflicting_payload_for_same_message_rejected_fail_closed
```
