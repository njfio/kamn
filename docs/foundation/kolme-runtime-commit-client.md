# Kolme Runtime Commit Adapter Contract (Issue #979)

This document captures the adapter-backed runtime commit client that maps
deterministic request payloads into provider calls with explicit typed failure
handling.

## Scope Delivered

- Added provider-facing adapter interfaces in `kamn-core`:
  - `KolmeRuntimeCommitProvider`
  - `KolmeRuntimeCommitProviderOutcome`
  - `KolmeRuntimeCommitProviderReceipt`
  - `KolmeRuntimeCommitProviderError`
- Added adapter-backed runtime commit client:
  - `AdapterBackedKolmeRuntimeCommitClient<P>`
- Added typed transport error classification:
  - `KolmeRuntimeCommitTransportErrorKind::{Timeout, Unavailable, MalformedResponse}`
- Extended runtime commit error contracts:
  - `KolmeRuntimeCommitError::ProviderTransport`
  - `KolmeRuntimeCommitError::ProviderMismatch`
  - `KolmeRuntimeCommitError::NonFinalReceipt`
- Added adapter integration coverage in:
  - `crates/kamn-core/tests/kolme_runtime_commit_client.rs`

## Deterministic Request Normalization Rules

- Adapter submissions call provider transport with:
  - canonical request payload from `KolmeRuntimeCommitRequest::to_wire_payload()`
  - deterministic idempotency key from `KolmeRuntimeCommitRequest::idempotency_key()`
- The adapter preserves validation semantics from
  `KolmeRuntimeCommitRequest::validate()` before provider dispatch.

## Provider and Finality Policy Rules

- `expected_provider` is mandatory and must be non-empty at client construction.
- Provider response must return matching `receipt.provider`.
- Provider response `receipt.commit_id` must be non-empty.
- Adapter mode requires `receipt.finality == Final`.
- `Pending` or `Failed` receipt finality is rejected as `NonFinalReceipt`.

## Typed Failure Mapping

- Provider timeout:
  - `KolmeRuntimeCommitProviderError::Timeout`
  - mapped to `KolmeRuntimeCommitError::ProviderTransport { kind: Timeout, ... }`
- Provider channel unavailable:
  - `KolmeRuntimeCommitProviderError::Unavailable { reason }`
  - mapped to `KolmeRuntimeCommitError::ProviderTransport { kind: Unavailable, ... }`
- Provider malformed response:
  - `KolmeRuntimeCommitProviderError::MalformedResponse { reason }`
  - mapped to `KolmeRuntimeCommitError::ProviderTransport { kind: MalformedResponse, ... }`
- Provider identity mismatch:
  - mapped to `KolmeRuntimeCommitError::ProviderMismatch`
- Non-final provider receipt:
  - mapped to `KolmeRuntimeCommitError::NonFinalReceipt`

## Validation Commands

Run targeted checks first:

```bash
cargo test -p kamn-core --test kolme_runtime_commit_client
cargo test -p kamn-core --test kolme_runtime_commit_finality
bash scripts/kolme/run_runtime_commit_contract_lane.sh
```

Then run broader regression:

```bash
cargo test -p kamn-core
```

## Regression Markers

- Mutated invalid runtime commit requests remain fail-closed (`Regression: #825`).
- Replay/tamper mismatch policy remains fail-closed (`Regression: #827`).
- Adapter provider mismatch/non-final receipts remain fail-closed (`Regression: #979`).
