# Trust Score Engine (Issue #212 / #213)

This document defines the deterministic trust score calculator for PRD Section 8.2 and its integration with persisted reputation state.

## PRD 8.2 Formula Mapping
Engine constants and components:

- `base_score = 500`
- `delivery_component = ((delivery_rate - 0.5) * 400.0) as i32`
- `response_component` buckets:
  - `0..=1000 => 100`
  - `1001..=5000 => 50`
  - `5001..=30000 => 0`
  - `>30000 => -50`
- `dispute_penalty = (dispute_rate * 150.0) as i32`
- `volume_bonus = (tasks_completed.min(1000) as f64 * 0.1) as i32`
- `endorsement_bonus = endorsements.len().min(50) as i32`

Final score formula:

`score = base_score + delivery_component + response_component - dispute_penalty + volume_bonus + endorsement_bonus`

## Deterministic Bounds and Versioning
- Engine version constant: `TRUST_SCORE_ENGINE_VERSION` (`v1-prd-8-2`).
- Raw score is clamped to `0..=1000`.
- Input validation is explicit and deterministic:
  - delivery_rate and dispute_rate must be within `0.0..=1.0`.
- `recalculate_and_persist_trust_score(...)` computes the score and writes it through `ReputationStore::set_trust_score(...)`, appending score history.

Regression guard:
- `1000ms` remains in the highest response bucket.

## Validation and Error Handling
- Invalid rate inputs return typed errors:
  - `InvalidDeliveryRate(f64)`
  - `InvalidDisputeRate(f64)`
- Persistence failures are propagated as `TrustScoreError::Reputation(...)`.

## Fast and Cost-Effective Validation
Use the targeted lane first:

```bash
cargo test -p kamn-core --test trust_score_engine --test trust_score_engine_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run full crate regression:

```bash
cargo test -p kamn-core
```
