# Key Management and Encryption Contract Rules

This document defines deterministic contract checks for key hierarchy rotation,
revocation invariants, and replay/stale-state rejection policies.

## Contract Scope
- Key role separation and rotation/revocation invariants.
- Recovery replay nonce rejection and stale-generation activation checks.
- Group sender-key replay/ratchet checks for stale-generation and nonce-replay rejection.
- Schema-validated evidence contract for key lifecycle policy drift detection.

## Evidence Schema
- `schema_version`: `kamn.key-lifecycle.invariant-evidence.v1`
- `evidence_key`: `key_lifecycle_invariant:<lane>:v1`
- `reason_key`: `key_lifecycle_invariant_reason:<final_decision>:v1`
- `final_decision`: `GO|NO-GO`
- Stable shell wrappers:
  - `scripts/message/generate_key_lifecycle_invariant_evidence_bundle.sh`
  - `scripts/message/check_key_lifecycle_invariant_policy.sh`
- Shared Python implementation:
  - `scripts/message/key_lifecycle_invariant_contract.py`

## Local Validation Commands
Run from repository root:

```bash
bash scripts/message/generate_key_lifecycle_invariant_evidence_bundle.sh --help
bash scripts/message/check_key_lifecycle_invariant_policy.sh --help
bash scripts/message/run_key_hierarchy_invariant_contract_lane.sh
bash scripts/message/test_generate_key_lifecycle_invariant_evidence_bundle.sh
bash scripts/message/test_run_key_hierarchy_invariant_contract_lane.sh
bash scripts/message/run_group_sender_replay_ratchet_contract_lane.sh
bash scripts/message/test_generate_group_sender_replay_ratchet_evidence_bundle.sh
bash scripts/message/test_run_group_sender_replay_ratchet_contract_lane.sh
cargo test -p kamn-core --test agent_key_hierarchy
cargo test -p kamn-core --test key_lifecycle
cargo test -p kamn-core --test key_recovery
cargo test -p kamn-core --test group_sender_keys
cargo test -p kamn-core --test group_sender_key_rotation_docs
cargo test -p kamn-core --test key_management_and_encryption_docs
```

## Policy Rules
- Replay/stale/revocation drift must force `NO-GO`.
- Group sender replay or stale-generation drift must force `NO-GO`.
- `report.reason_codes` must be sorted deterministic strings.
- `final_decision` must match derived policy from report booleans.
- `evidence_key` and `reason_key` must match deterministic lane/decision keys.

## Kolme Live Signer Provider Contracts
- Production signer secret loading is provider-backed (`KolmeLiveSignerSecretProvider`)
  with explicit fail-closed checks before private-key material is read.
- Strict signer profiles remain bounded to `ops-primary` and `ops-secondary`.
- Supported strict key source remains `env-local` for node runtime signer loading.
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK` must remain unset in
  production signer flows; presence triggers deterministic
  `fallback_signer_secret_present_violation` rejection.
- Profile-selected signer key env contracts remain explicit:
  - `ops-primary`: `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX`
  - `ops-secondary`: `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY`

## Regression Marker
- replay/stale key activation drift is rejected (`Regression: #931`)
