# Group Sender-Key Replay and Ratchet Contract Rules

This document defines deterministic contract checks for sender-key replay rejection,
ratchet generation monotonicity, and stale-generation fail-closed behavior.

## Contract Scope
- Sender-key replay and stale-generation rejection evidence contracts.
- Deterministic reason-code and evidence key validation for policy drift detection.
- Bounded runtime contract lane for fast and cost-effective CI checks.

## Evidence Schema
- `schema_version`: `kamn.group-sender.replay-ratchet-evidence.v1`
- `evidence_key`: `group_sender_replay_ratchet:<lane>:v1`
- `reason_key`: `group_sender_replay_ratchet_reason:<final_decision>:v1`
- `final_decision`: `GO|NO-GO`
- Stable shell wrappers:
  - `scripts/message/generate_group_sender_replay_ratchet_evidence_bundle.sh`
  - `scripts/message/check_group_sender_replay_ratchet_policy.sh`
- Shared Python implementation:
  - `scripts/message/group_sender_replay_ratchet_contract.py`

## Local Validation Commands
Run from repository root:

```bash
bash scripts/message/generate_group_sender_replay_ratchet_evidence_bundle.sh --help
bash scripts/message/check_group_sender_replay_ratchet_policy.sh --help
bash scripts/message/run_group_sender_replay_ratchet_contract_lane.sh
bash scripts/message/test_generate_group_sender_replay_ratchet_evidence_bundle.sh
bash scripts/message/test_run_group_sender_replay_ratchet_contract_lane.sh
cargo test -p kamn-core --test group_sender_keys
cargo test -p kamn-core --test docs_contract_matrix_wave2_harness
```

## Policy Rules
- Any replay nonce detection must force `NO-GO`.
- Any stale-generation payload detection must force `NO-GO`.
- Any signature tamper detection must force `NO-GO`.
- `report.reason_codes` must be sorted deterministic strings.
- `final_decision` must match derived policy from report booleans.
- Lane runtime must remain within `GROUP_SENDER_REPLAY_RATCHET_MAX_SECONDS`.

## Regression Marker
- stale-generation and nonce replay payloads are rejected (`Regression: #932`)
