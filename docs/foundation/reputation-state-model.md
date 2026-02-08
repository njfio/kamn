# Reputation State Model and Persistence (Issue #214 / #215)

This document captures the first implementation slice of the PRD Section 8 reputation system: state shape, persistence contract, and deterministic validation behavior.

## PRD 8.1 Metrics Coverage
`AgentReputation` persists the PRD core metrics:

- `trust_score` (0-1000)
- `delivery_rate`
- `response_time_avg_ms`
- `dispute_rate`
- `tasks_completed`, `tasks_failed`, `tasks_delegated`
- `total_earned`, `total_spent`
- `endorsements`
- `disputes`
- `verified_capabilities`
- `last_updated_block`
- `score_history`

The initial default score is `500` and history starts with an initial snapshot at registration.

## Persistence Contract
- State namespace: `kamn.reputation.scores`
- Canonical key shape: `kamn.reputation.scores:agent:<method-specific-id>`
- Persisted record payload:
  - canonical `state_key`
  - `state_version`
  - full `AgentReputation` snapshot

`ReputationStore::export_records()` returns records sorted by canonical state key for deterministic persistence ordering.
`ReputationStore::restore_from_records(...)` enforces:
- state-version compatibility
- state-key and DID consistency
- duplicate state-key rejection

## Validation and Error Handling
- Invalid agent DID values are rejected on registration and all attestation/capability paths.
- Empty IDs/reasons/notes/proof references are rejected with explicit field-specific errors.
- Invalid or zero block heights are rejected.
- Duplicate endorsement IDs and duplicate dispute IDs are rejected.
- Duplicate capability verification entries for the same `(capability, verifier)` pair are rejected.
- Trust score updates reject values above 1000.
- Missing response time is rejected for `Completed` and `Failed` task outcomes.

Trust score boundary checks are inclusive for `1000`.

## Fast and Cost-Effective Validation
Use the targeted lane first:

```bash
cargo test -p kamn-core --test reputation_state_model --test reputation_state_model_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run regression coverage for the crate:

```bash
cargo test -p kamn-core
```
