# Message Lifecycle and Index Queries (Issue #114)

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
- Unknown message IDs return `MessageLifecycleError::NotFound`.

## Local Validation
Run from repository root:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test message_lifecycle_queries
cargo test -p kamn-core
```

## Notes
This slice keeps indexes in-memory and deterministic (`BTreeMap`/`BTreeSet`)
so lifecycle audit behavior can be validated quickly at low CI cost.
