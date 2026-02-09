# Message Delivery Guards (Issue #116)

This document defines the first implementation slice for nonce, TTL, replay,
and failed-delivery notification controls.

## Core Types
- `DeliveryGuardInput`: delivery metadata (`message_id`, sender/recipient, nonce, created/expires, received_at).
- `MessageDeliveryGuards`: in-memory nonce + replay guard state.
- `DeliveryValidationResult`: `Accepted` or `Rejected(FailedDeliveryNotice)`.
- `DeliveryFailureCode`:
  - `NonceOutOfSequence { expected, found }`
  - `Replay`
  - `Expired`
  - `InvalidWindow`
- `FailedDeliveryNotice`: deterministic signed failure artifact for rejected deliveries.

## Validation Rules
- Reject if `expires <= created` with `InvalidWindow`.
- Reject if `received_at > expires` with `Expired`.
- Reject if `message_id` was already accepted with `Replay`.
- Reject if `nonce` does not match sender expected nonce with `NonceOutOfSequence`.
- Accept only when all checks pass; accepted deliveries:
  - record `message_id` in replay set
  - advance sender nonce (`expected_nonce = nonce + 1`)

## Failure Notifications
- Rejections emit `FailedDeliveryNotice` with deterministic signature format:
  `notice:<message_id>:<code>:<recipient>:<received_at>:<nonce>`
- Rejected deliveries do not mutate nonce progression state.

## Durable Snapshot Stores (Issue #701)
- `DeliveryGuardSnapshot` is schema-versioned and validates sender/nonce/message-id integrity on restore.
- `DurableGuardSnapshotBundle::capture` and `restore_into` persist and restore delivery + channel policy guard state atomically.
- `InMemoryDurableGuardSnapshotStore` and `FileDurableGuardSnapshotStore` provide deterministic save/load contracts.
- Truncated/corrupted durable bundle payloads fail closed (`Regression: #679`).

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test message_delivery_guards
cargo test -p kamn-core --test message_delivery_guards_docs
cargo test -p kamn-core --test durable_guard_snapshot_store
bash scripts/guard/run_durable_guard_recovery_contract_lane.sh
cargo test -p kamn-core
```

## Notes
Guard validation remains deterministic and dependency-free while durable snapshot
stores provide restart-safe persistence contracts with low-cost CI coverage.
