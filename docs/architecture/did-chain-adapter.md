# DID Chain Adapter Architecture

This document captures the DID chain-adapter flow delivered for roadmap Phase 4.2 core implementation (Task #2936, Subtask #2937).

## Scope

- Deterministic chain-submission contracts for DID registration and lifecycle mutation.
- Retry classification and finality tracking for lifecycle mutation submissions.
- Kolme-backed lifecycle adapter path that reuses runtime-commit client contracts.

## Core Types

- `DidRegistrationChainAdapter`
  - Handles registration submission via `submit_registration`.
- `DidLifecycleChainAdapter`
  - Handles lifecycle mutation submission via `submit_lifecycle_mutation`.
- `DidLifecycleChainSubmissionRequest`
  - Canonical request envelope:
    - `did`
    - `actor_did`
    - `nonce`
    - `action`
    - `idempotency_key`
    - `payload_hash`
- `DidLifecycleChainSubmissionResult`
  - Result envelope with `retry_class`, `outcome`, and mutation `evidence`.

## Registry Flow

Entry points in `crates/kamn-core/src/did_registry.rs`:

- `idempotency_key_for_lifecycle_mutation(request)`
- `submit_lifecycle_mutation_via_chain_adapter(adapter, request)`
- `record_lifecycle_finality(did, nonce, idempotency_key, sequence, status, receipt)`
- `lifecycle_finality(did, nonce)`

Behavior:

1. Compute deterministic lifecycle idempotency key from DID, actor, nonce, action, and action payload fingerprint.
2. Classify retry posture for `(did, nonce)`:
   - `NewSubmission`
   - `RetryableInFlight`
   - `FinalizedNoRetry`
   - `ConflictNoRetry`
3. Apply mutation exactly once on `NewSubmission`, store deterministic evidence, and fail closed on nonce/key conflicts.
4. Submit lifecycle request to adapter unless already finalized (`FinalizedNoOp`).

## Kolme Adapter

- `KolmeDidLifecycleChainAdapter<C>`
  - Generic over `KolmeRuntimeCommitClient`.
  - Converts lifecycle request into deterministic `KolmeRuntimeCommitRequest`.
  - Maps runtime-commit outcomes to DID chain outcomes:
    - `Submitted` -> `Submitted(receipt)`
    - `Duplicate` -> `Duplicate(receipt)`
    - `Rejected` -> `Rejected { reason }`
  - Maps runtime-commit errors to `DidRegistryError::ChainAdapterSubmitFailed`.

## Validation Commands

```bash
cargo test -p kamn-core --test did_registry_transactions
cargo test -p kamn-core --test did_registry_transactions_docs
cargo clippy -p kamn-core -- -D warnings
cargo fmt --check
```
