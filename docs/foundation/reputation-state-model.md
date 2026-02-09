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

## Deterministic Reputation Dispute Evidence Contract (Issue #738)
Dispute adjudication uses machine-verifiable bundles so resolution outcomes stay reproducible and tamper-evident.

- Schema contract:
  - `schema_version`: `kamn.reputation.dispute-evidence.v1`
  - `reason_key`: `reputation_dispute_reason_codes:<final_decision>:v1`
  - `reason_codes`: sorted deterministic policy failure codes
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_dispute_evidence_bundle.sh --output-file /tmp/reputation-dispute.json --dispute-id dispute-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --dispute-reason-code QUALITY --evidence-uri s3://kamn-audit/reputation/dispute-001.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --evidence-hash-verified PASS --original-trust-score 640 --proposed-trust-score 560 --max-adjustment-points 120 --policy-window-open true --approval-recorded true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_dispute_policy.sh --bundle-file /tmp/reputation-dispute.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_dispute_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_reputation_dispute_deep_lane.sh --output-json reputation-dispute-report.json`
- Replay matrix runner:
  - `python3 scripts/reputation/run_reputation_dispute_matrix.py --fixture fixtures/reputation_dispute/replay_cases.json --output-json reputation-dispute-report.json`
- Runtime budget control:
  - `REPUTATION_DISPUTE_MAX_SECONDS` (contract lane fails closed when runtime exceeds budget)
- Regression policy:
  - tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`).
  - reason-code mismatch or tampered dispute evidence payloads force `NO-GO` (`Regression: #934`).

## Deterministic Reputation Signal Quarantine Contract (Issue #935)
Signal ingestion is gated by deterministic quarantine checks before any reputation state mutation is accepted.

- Schema contract:
  - `schema_version`: `kamn.reputation.signal-quarantine-evidence.v1`
  - `reason_key`: `reputation_signal_quarantine_reason_codes:<final_decision>:v1`
  - `reason_codes`: sorted deterministic quarantine failure codes
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_signal_quarantine_evidence_bundle.sh --output-file /tmp/reputation-signal-quarantine.json --lane contract --signal-id signal-001 --subject-did did:kamn:agent-001 --signal-kind ENDORSEMENT --source-channel TELEGRAM --event-age-seconds 30 --payload-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --payload-signature-verified PASS --nonce-unique true --rate-within-threshold true --source-attested true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_signal_quarantine_policy.sh --bundle-file /tmp/reputation-signal-quarantine.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_signal_quarantine_contract_lane.sh`
- Runtime budget control:
  - `REPUTATION_QUARANTINE_MAX_SECONDS` (contract lane fails closed when runtime exceeds budget)
- Regression policy:
  - tampered quarantine reason codes, replayed nonces, and stale signal payloads force `NO-GO` quarantine (`Regression: #935`).

## Deterministic Reputation Recovery Reversal Contract (Issue #936)
False-positive penalties require deterministic reversal checks so recovery remains auditable and never drifts into irreversible state.

- Schema contract:
  - `schema_version`: `kamn.reputation.recovery-reversal-evidence.v1`
  - `reason_key`: `reputation_recovery_reason_codes:<final_decision>:v1`
  - `reason_codes`: sorted deterministic recovery failure codes
  - `recovery_action`: `REVERSE_PENALTY` (GO) or `HOLD_PENALTY` (NO-GO)
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_recovery_evidence_bundle.sh --output-file /tmp/reputation-recovery.json --lane contract --recovery-id recovery-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --pre-penalty-trust-score 700 --post-penalty-trust-score 540 --proposed-recovered-trust-score 660 --max-reversal-points 160 --false-positive-confirmed true --reviewer-quorum-satisfied true --audit-evidence-verified PASS --replay-guard-pass true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_recovery_policy.sh --bundle-file /tmp/reputation-recovery.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_recovery_contract_lane.sh`
- Runtime budget control:
  - `REPUTATION_RECOVERY_MAX_SECONDS` (contract lane fails closed when runtime exceeds budget)
- Regression policy:
  - false-positive irreversible-penalty paths, replayed recovery nonces, and tampered recovery reason codes force `NO-GO` (`Regression: #936`).

## Weighted Decay and Anti-Gaming Threshold Contract (Issue #736)
Trust-score updates apply deterministic weighted decay windows and typed abuse-threshold penalties before score persistence.

- Compact PR lane entrypoint:
  - `bash scripts/reputation/run_weighted_decay_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_weighted_decay_deep_lane.sh --output-json reputation-weighted-decay-report.json`
- Compact fixture matrix:
  - `python3 scripts/reputation/run_weighted_decay_matrix.py --fixture fixtures/reputation_decay/compact_cases.json --output-json reputation-weighted-decay-report.json`
- Deep adversarial fixture matrix:
  - `python3 scripts/reputation/run_weighted_decay_matrix.py --fixture fixtures/reputation_decay/adversarial_cases.json --output-json reputation-weighted-decay-report.json`
- Regression policy:
  - replayed reciprocity, burst-spam, and churn abuse fixtures remain penalized (`Regression: #730`).

## Fast and Cost-Effective Validation
Use the targeted lane first:

```bash
cargo test -p kamn-core --test reputation_state_model --test reputation_state_model_docs
bash scripts/reputation/test_generate_reputation_dispute_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_dispute_contract_lane.sh
bash scripts/reputation/test_generate_reputation_signal_quarantine_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_signal_quarantine_contract_lane.sh
bash scripts/reputation/test_check_reputation_signal_quarantine_policy.sh
bash scripts/reputation/test_generate_reputation_recovery_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_recovery_contract_lane.sh
bash scripts/reputation/test_check_reputation_recovery_policy.sh
bash scripts/reputation/test_run_reputation_dispute_matrix.sh
bash scripts/reputation/test_run_weighted_decay_contract_lane.sh
bash scripts/reputation/test_run_weighted_decay_matrix.sh
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run regression coverage for the crate:

```bash
bash scripts/reputation/test_run_reputation_dispute_deep_lane.sh
bash scripts/reputation/test_run_weighted_decay_deep_lane.sh
cargo test -p kamn-core
```
