# Tasks — Issue #4820

- [x] T1 (Red): add failing migration conformance test `scripts/framework/test_common_shell_migration_wave_evidence_bundle.sh` and capture violation output before migration.
- [x] T2 (Green): migrate 25 evidence-bundle test scripts to source `scripts/lib/common.sh`.
- [x] T3 (Refactor): remove local duplicated `ROOT_DIR`, `extract_value`, and `assert_eq` helper definitions from migrated scripts.
- [x] T4 (Verify): run migration conformance test and full migrated suite; update issue process log with evidence.

## Verification Evidence

- RED:
  - `bash scripts/framework/test_common_shell_migration_wave_evidence_bundle.sh` -> failed with 97 violations before migration
- GREEN:
  - `bash scripts/framework/test_common_shell_migration_wave_evidence_bundle.sh`
  - run all migrated scripts:
    - `scripts/bridge/test_generate_bridge_adapter_conformance_evidence_bundle.sh`
    - `scripts/bridge/test_generate_bridge_replay_redaction_evidence_bundle.sh`
    - `scripts/bridge/test_generate_cross_chain_outbound_intent_evidence_bundle.sh`
    - `scripts/bridge/test_generate_localhost_bridge_demo_evidence_bundle.sh`
    - `scripts/canary/test_generate_post_cutover_slo_evidence_bundle.sh`
    - `scripts/channel/test_generate_channel_retention_redaction_evidence_bundle.sh`
    - `scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh`
    - `scripts/compliance/test_generate_soc2_control_evidence_bundle.sh`
    - `scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh`
    - `scripts/did/test_generate_federated_did_handshake_evidence_bundle.sh`
    - `scripts/did/test_generate_lifecycle_operator_binding_evidence_bundle.sh`
    - `scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh`
    - `scripts/governance/test_generate_governance_simulation_evidence_bundle.sh`
    - `scripts/governance/test_generate_stake_slash_risk_evidence_bundle.sh`
    - `scripts/kolme/test_generate_fork_compatibility_evidence.sh`
    - `scripts/message/test_generate_group_sender_replay_ratchet_evidence_bundle.sh`
    - `scripts/message/test_generate_key_lifecycle_invariant_evidence_bundle.sh`
    - `scripts/message/test_generate_processor_proof_artifact_evidence_bundle.sh`
    - `scripts/reputation/test_generate_weighted_decay_property_evidence_bundle.sh`
    - `scripts/runtime/test_generate_live_network_pilot_artifact_summary.sh`
    - `scripts/sdk/test_generate_live_transport_replay_tamper_evidence_bundle.sh`
    - `scripts/signer/test_generate_secure_provider_key_lifecycle_evidence_bundle.sh`
    - `scripts/task/test_generate_federated_delegation_settlement_evidence_bundle.sh`
    - `scripts/token/test_generate_token_launch_handoff_evidence_bundle.sh`
    - `scripts/treasury/test_generate_treasury_disbursement_evidence_bundle.sh`
