# Message Lifecycle and Index Queries (Issue #114 / #563)

This document defines the first implementation slice for message lifecycle
state transitions and deterministic index query APIs.

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

## Validation Rules
- `message_id` must be non-empty and unique.
- `sender` and each recipient must be valid `kamn:did:agent:*` DIDs.
- Recipients must be non-empty.
- `created` and `expires` timestamps must be non-empty, with `expires > created`.
- `observed_at` passed to expiry APIs must be non-empty.
- Unknown message IDs return `MessageLifecycleError::NotFound`.
- Expiry APIs only transition active records (`Created`, `Signed`, `Broadcast`,
  `Included`, `Delivered`) to `Expired`.

## Processor Proof-Gated Validation
- `validate_with_processor_proof(message_id, expected_payload_commitment, artifact, evaluator)` gates lifecycle validation on deterministic processor proof admission.
- The message must already be in `Delivered` state before proof-gated validation is attempted.
- On successful proof admission, the lifecycle transition advances `Delivered -> Validated`.
- Proof admission errors are surfaced as `MessageProofAdmissionError::Proof(...)` and do not mutate lifecycle status.
- tampered proof artifacts must not advance message state (`Regression: #510`).

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test message_lifecycle_queries
cargo test -p kamn-core --test message_lifecycle_proof_admission --test message_lifecycle_docs
cargo test -p kamn-core
```

## Notes
This slice keeps indexes in-memory and deterministic (`BTreeMap`/`BTreeSet`)
so lifecycle audit behavior can be validated quickly at low CI cost.
