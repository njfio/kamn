# DID Registry Transactions (Issues #110, #685, #2936)

This document defines the first implementation slice for DID
register/resolve/update/revoke transaction behavior.

## Behavior
- `register(did, document)` stores a DID document for a new DID.
- Registering an already active DID returns `DidRegistryError::AlreadyRegistered`.
- Registering a revoked DID returns `DidRegistryError::Revoked`.
- `resolve(did)` returns the active DID document.
- Resolving a revoked DID returns `DidRegistryError::Revoked`.
- Resolving an unknown DID returns `DidRegistryError::NotFound`.
- `update(did, document)` replaces the active DID document for an existing DID.
- Updating a revoked DID returns `DidRegistryError::Revoked`.
- Updating an unknown DID returns `DidRegistryError::NotFound`.
- `revoke(did)` transitions an active DID to revoked.
- Re-registering a revoked DID is rejected (`DidRegistryError::Revoked`) as a regression guard.
- `idempotency_key_for_register(did, document)` derives a deterministic submission key.
- `submit_registration_via_chain_adapter(adapter, did, document)` executes DID submission through typed chain-adapter outcomes.
- `register_with_retry_guard(did, document)` classifies retries as:
  - `NewSubmission`
  - `RetryableInFlight`
  - `FinalizedNoRetry`
  - `ConflictNoRetry`
- chain adapter typed outcomes:
  - `Submitted(receipt)`
  - `Duplicate(receipt)`
  - `Rejected { reason }`
  - `FinalizedNoOp`
- `record_register_finality(did, key, sequence, status, receipt)` enforces finality update safety:
  - duplicate update with identical sequence/status/receipt is idempotent
  - stale sequence is rejected (`DidRegistryError::StaleFinalityUpdate`)
  - conflicting update at same sequence is rejected (`DidRegistryError::ConflictingFinalityUpdate`)
  - unknown key is rejected (`DidRegistryError::UnknownSubmissionIdempotencyKey`)
- `apply_lifecycle_mutation(request)` enforces deterministic DID mutation transactions for:
  - `DidLifecycleMutationAction::Rotate { document }`
  - `DidLifecycleMutationAction::Revoke`
  - `DidLifecycleMutationAction::Recover { document }`
- `DidLifecycleMutationRequest` includes `did`, `actor_did`, `nonce`, and `action`.
- Successful mutation emits `DidLifecycleMutationEvidence` with
  `reason_code=did_lifecycle_mutation_allowed`.
- `idempotency_key_for_lifecycle_mutation(request)` derives deterministic lifecycle submission keys.
- `submit_lifecycle_mutation_via_chain_adapter(adapter, request)` executes lifecycle submission with retry classes:
  - `NewSubmission`
  - `RetryableInFlight`
  - `FinalizedNoRetry`
  - `ConflictNoRetry`
- lifecycle chain adapter request contract:
  - `DidLifecycleChainSubmissionRequest { did, actor_did, nonce, action, idempotency_key, payload_hash }`
- lifecycle finality tracking:
  - `record_lifecycle_finality(did, nonce, key, sequence, status, receipt)`
  - `lifecycle_finality(did, nonce)`
- Kolme-backed lifecycle integration path:
  - `KolmeDidLifecycleChainAdapter`
  - maps lifecycle mutation submissions into deterministic `KolmeRuntimeCommitRequest` payloads
  - maps provider outcomes back into DID chain submission outcomes
- Lifecycle mutation reason codes are deterministic:
  - `did_lifecycle_mutation_nonce_invalid`
  - `did_lifecycle_mutation_nonce_replay`
  - `did_lifecycle_mutation_unauthorized_actor`
  - `did_lifecycle_mutation_invalid_transition`
  - `did_chain_adapter_submit_failed`

## Validation Rules
- The DID document `id` must match the target `did`.
- Document mismatch is rejected with `DidRegistryError::DocumentDidMismatch`.
- retry classification and finality checks are deterministic across duplicate/stale/conflict outcomes (`Regression: #678`).
- chain adapter submission contract remains deterministic through `InMemoryDidRegistrationChainAdapter` for low-cost CI verification.
- Lifecycle mutation nonce replay, unauthorized actor mutation, and invalid revoke/recover transitions fail closed (`Regression: #889`).
- Lifecycle mutation submission rejects conflicting payloads for same DID+nonce idempotency window (`Regression: #2936`).

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test did_registry_transactions
cargo test -p kamn-core --test did_registry_transactions -- retry_classification_is_deterministic_for_duplicate_submission
cargo test -p kamn-core --test did_registry_transactions -- functional_chain_submission_adapter_returns_typed_submitted_outcome
cargo test -p kamn-core --test did_registry_transactions -- integration_chain_submission_adapter_deduplicates_retry_outcomes
cargo test -p kamn-core --test did_registry_transactions -- regression_chain_submission_adapter_exposes_rejected_outcome_without_panicking
cargo test -p kamn-core --test did_registry_transactions -- regression_register_finality_rejects_stale_or_conflicting_updates
cargo test -p kamn-core --test did_registry_transactions -- functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code
cargo test -p kamn-core --test did_registry_transactions -- integration_lifecycle_revoke_then_recover_restores_active_resolution
cargo test -p kamn-core --test did_registry_transactions -- regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed
cargo test -p kamn-core --test did_registry_transactions -- functional_lifecycle_chain_submission_through_kolme_adapter_returns_typed_outcome
cargo test -p kamn-core --test did_registry_transactions -- integration_lifecycle_chain_submission_allows_retry_without_reapplying_mutation
cargo test -p kamn-core --test did_registry_transactions -- regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload
bash scripts/did/run_did_registry_contract_lane.sh
cargo test -p kamn-core
```

## Notes
This slice is intentionally in-memory and deterministic so transaction logic can be
validated quickly with low CI cost before introducing persistent storage.
