# Trust Score Engine (Issue #212 / #213 / #735 / #736)

This document defines deterministic trust-score scoring, weighted decay windows, and anti-gaming abuse-threshold penalties for PRD Section 8.2 production hardening.

## PRD 8.2 Formula Mapping
Engine constants and core components:

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

## Weighted Decay Windows and Abuse Threshold Mapping
- Decay windows:
  - recent activity window: `<= 128` blocks
  - mid activity window: `<= 512` blocks
  - stale history window: `> 512` blocks
- Weighted decay applies a deterministic `decay_multiplier_bps` to `volume_bonus` and `endorsement_bonus`:
  - `decayed_volume_bonus = volume_bonus * decay_multiplier_bps / 1000`
  - `decayed_endorsement_bonus = endorsement_bonus * decay_multiplier_bps / 1000`
- Stale-only history entries do not increase `decay_multiplier_bps`; only recent and mid windows can lift the multiplier.
- Abuse-threshold penalties map to typed outcomes:
  - `ReciprocityRing` when delegation ratio breaches threshold
  - `BurstSpam` when failure-burst ratio breaches threshold
  - `ChurnSpike` when dispute churn ratio breaches threshold
  - `Compound` when multiple abuse thresholds breach together
- Final score formula:
  - `score = base_score + delivery_component + response_component - dispute_penalty + decayed_volume_bonus + decayed_endorsement_bonus - abuse_penalty_points`

## Deterministic Bounds and Versioning
- Engine version constant: `TRUST_SCORE_ENGINE_VERSION` (`v2-prd-8-2-anti-gaming`).
- Raw score is clamped to `0..=1000`.
- Input validation is explicit and deterministic:
  - delivery_rate and dispute_rate must be within `0.0..=1.0`.
- `recalculate_and_persist_trust_score(...)` computes the score and writes it through `ReputationStore::set_trust_score(...)`, appending score history.

Regression guards:
- `1000ms` remains in the highest response bucket.
- replayed reciprocity/burst/churn abuse fixtures remain penalized (`Regression: #730`).
- stale-only history does not improve decay multiplier (`Regression: #768`).

## Weighted Decay and Anti-Gaming Fixture Lanes (Issue #736)
- Compact PR lane entrypoint:
  - `bash scripts/reputation/run_weighted_decay_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_weighted_decay_deep_lane.sh --output-json reputation-weighted-decay-report.json`
- Fixture matrix runner:
  - `python3 scripts/reputation/run_weighted_decay_matrix.py --fixture fixtures/reputation_decay/compact_cases.json --output-json reputation-weighted-decay-report.json`

## Validation and Error Handling
- Invalid rate inputs return typed errors:
  - `InvalidDeliveryRate(f64)`
  - `InvalidDisputeRate(f64)`
- Persistence failures are propagated as `TrustScoreError::Reputation(...)`.

## Fast and Cost-Effective Validation
Use the targeted lane first:

```bash
cargo test -p kamn-core --test trust_score_engine --test trust_score_engine_docs
bash scripts/reputation/test_run_weighted_decay_contract_lane.sh
bash scripts/reputation/test_run_weighted_decay_matrix.sh
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run full crate regression:

```bash
bash scripts/reputation/test_run_weighted_decay_deep_lane.sh
cargo test -p kamn-core
```
