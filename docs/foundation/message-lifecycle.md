# Message Lifecycle and Snapshot Persistence Contracts (Issue #114 / #563 / #617)

This document defines lifecycle transitions, deterministic index APIs, and
durable snapshot persistence/restore guards for message lifecycle state.

## Lifecycle State Machine
- `Created -> Signed -> Broadcast -> Included -> Delivered -> Validated -> Rejected -> Expired`
- Any transition outside the canonical chain is rejected with
  `MessageLifecycleError::InvalidTransition`.

## APIs
- `register(message_id, sender, recipients, created, expires)`:
  validates envelope identifiers and initializes status as `Created`.
- `transition(message_id, status)`:
  applies legal state transitions and updates indexes.
- `status(message_id)`:
  returns current lifecycle status.
- `expire_message_if_overdue(message_id, observed_at)`:
  expires an active message deterministically when `observed_at > expires`.
- `expire_overdue_messages(observed_at)`:
  sweeps active records and returns deterministically ordered expired message IDs.
- `ids_by_status(status)`:
  returns deterministic message IDs for a lifecycle stage.
- `ids_by_sender(sender)`:
  returns deterministic message IDs sent by a DID.
- `ids_by_recipient(recipient)`:
  returns deterministic message IDs addressed to a DID.
- `export_snapshot()`:
  exports deterministic state as `MessageLifecycleSnapshot`.
- `restore_snapshot(snapshot)`:
  validates and restores snapshot state/indexes atomically.
- `MessageLifecycleSnapshotStore`:
  durable persistence contract with `write(snapshot)` and `read_latest()`.
- `FileMessageLifecycleSnapshotStore::recover_latest_and_repair()`:
  loads latest snapshot and truncates corrupted payloads deterministically.

## Validation Rules
- `message_id` must be non-empty and unique.
- `sender` and each recipient must be valid `kamn:did:agent:*` DIDs.
- Recipients must be non-empty.
- `created` and `expires` timestamps must be non-empty, with `expires > created`.
- `observed_at` passed to expiry APIs must be non-empty.
- Unknown message IDs return `MessageLifecycleError::NotFound`.
- Expiry APIs only transition active records (`Created`, `Signed`, `Broadcast`,
  `Included`, `Delivered`) to `Expired`.

## Snapshot Persistence and Restore Contract Rules
- Snapshot schema is versioned with
  `MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION`.
- Snapshot record validation enforces lifecycle envelope constraints used by
  `register(...)`.
- Restore guards reject:
  - duplicate message IDs in payload
  - empty history
  - history that does not begin at `Created`
  - invalid transition chains inside history
  - status/history mismatch
- File-backed persistence uses deterministic record lines:
  - `schema|<version>`
  - `record|<message_id>|<sender>|<recipient_csv>|<created>|<expires>|<status_code>|<history_code_csv>`
- Delimiter poisoning (`|`, newline, and invalid commas in non-list fields) is
  rejected during serialization.
- Corrupted payload recovery truncates invalid on-disk data and returns
  `latest=None` with `repaired=true`.
- Regression contract:
  - stale/invalid snapshot payload and metadata are rejected (`Regression: #617`)
  - duplicate message IDs on restore are rejected (`Regression: #617`)
  - status/history mismatch on restore is rejected (`Regression: #617`)

## Processor Proof-Gated Validation
- `validate_with_processor_proof(message_id, expected_payload_commitment, artifact, evaluator)` gates lifecycle validation on deterministic processor proof admission.
- The message must already be in `Delivered` state before proof-gated validation is attempted.
- On successful proof admission, the lifecycle transition advances `Delivered -> Validated`.
- Proof admission errors are surfaced as `MessageProofAdmissionError::Proof(...)` and do not mutate lifecycle status.
- tampered proof artifacts must not advance message state (`Regression: #510`).

## Fast and Cost-Effective Validation
Run targeted checks from repository root:

```bash
cargo test -p kamn-core message_lifecycle::tests::
cargo test -p kamn-core --test message_lifecycle_queries
cargo test -p kamn-core --test message_lifecycle_proof_admission
cargo test -p kamn-core --test message_lifecycle_docs
bash scripts/message/run_message_lifecycle_contract_lane.sh
```

Scheduled deep-lane command:

```bash
cargo test -p kamn-core message_lifecycle::tests::performance_message_lifecycle_snapshot_deep_lane_stress -- --ignored
bash scripts/message/run_message_lifecycle_deep_lane.sh
```

Then run strict gates:

```bash
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```
