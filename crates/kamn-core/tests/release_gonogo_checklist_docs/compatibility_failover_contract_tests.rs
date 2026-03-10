use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_FAILOVER_SYNC_DRILL_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Failover + Sync Drill Evidence Contract",
    "select_failover_sync_drill_lane.sh",
    "run_failover_sync_drill_preflight_contract_lane.sh",
    "run_failover_sync_drill_deep_lane.sh",
    "run_failover_sync_drill_suite.sh",
];

#[test]
fn checklist_contains_failover_sync_drill_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_FAILOVER_SYNC_DRILL_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_failover_sync_drill_evidence_contract");
}

const CHECKLIST_CONTAINS_FORK_CHOICE_FINALITY_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS: &[&str] = &[
    "## Fork-Choice Finality Taxonomy and Runbook Marker Parity Gate (Issues #4252, #4257, #4258)",
    "check_libp2p_convergence_process_isolated_live_policy.sh --report-file /tmp/libp2p-convergence-process-isolated-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/libp2p-convergence-process-isolated-live-policy.json",
    "validate_libp2p_convergence_process_isolated_live_contract_lane.sh",
    "finality_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1",
    "convergence_reason_codes_csv=fork_choice_stale_block_height",
    "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1",
    "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "finality_taxonomy_runbook_reason_code=none|<reason>",
    "finality_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "Regression: #4257",
    "Regression: #4258",
];

#[test]
fn checklist_contains_fork_choice_finality_taxonomy_runbook_parity_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_FORK_CHOICE_FINALITY_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS, "checklist_contains_fork_choice_finality_taxonomy_runbook_parity_gate");
}

const CHECKLIST_CONTAINS_PEER_ADAPTER_REASON_PROJECTION_MULTI_PROCESS_GATE_MARKERS: &[&str] = &[
    "## Peer Adapter Reason Projection and Multi-Process Validation Hooks (Issue #4320)",
    "cargo test -p kamn-core --test p2p_peer_adapter_reason_projection",
    "validate_libp2p_convergence_process_isolated_live.sh",
    "check_libp2p_convergence_process_isolated_live_policy.sh",
    "validate_libp2p_convergence_process_isolated_live_contract_lane.sh",
    "peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1",
    "peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer",
    "peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout",
    "peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted",
    "peer_adapter_multi_process_validation_local_heavy_status=required",
    "Regression: #4320",
];

#[test]
fn checklist_contains_peer_adapter_reason_projection_multi_process_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_PEER_ADAPTER_REASON_PROJECTION_MULTI_PROCESS_GATE_MARKERS, "checklist_contains_peer_adapter_reason_projection_multi_process_gate");
}

const CHECKLIST_CONTAINS_LIVE_NETWORK_PILOT_LAUNCH_AND_ROLLBACK_EVIDENCE_GATES_MARKERS: &[&str] = &[
    "## Live-Network Pilot Launch and Rollback Evidence Gates",
    "run_live_network_smoke_lane.sh",
    "run_live_network_pilot_deep_lane.sh",
    "check_live_network_pilot_artifact_summary_policy.sh",
    "run_live_network_pilot_deep_contract_lane.sh",
    "select_live_network_partition_reconnect_lane.sh",
    "run_live_network_partition_reconnect_smoke_lane.sh",
    "run_live_network_partition_reconnect_deep_lane.sh",
    "check_live_network_partition_reconnect_policy.sh",
    "run_live_network_partition_reconnect_contract_lane.sh",
    "fixtures/runtime/live_network_partition_reconnect_matrix_cases.json",
];

#[test]
fn checklist_contains_live_network_pilot_launch_and_rollback_evidence_gates() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_LIVE_NETWORK_PILOT_LAUNCH_AND_ROLLBACK_EVIDENCE_GATES_MARKERS, "checklist_contains_live_network_pilot_launch_and_rollback_evidence_gates");
}

const CHECKLIST_CONTAINS_WATCHDOG_PROOF_CONSENSUS_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Validator/Watchdog Proof Consensus Evidence Contract",
    "run_watchdog_proof_consensus_contract_lane.sh",
    "run_watchdog_proof_consensus_deep_lane.sh",
    "generate_watchdog_proof_consensus_evidence_bundle.sh",
    "check_watchdog_proof_consensus_policy.sh",
    "KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE",
];

#[test]
fn checklist_contains_watchdog_proof_consensus_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_WATCHDOG_PROOF_CONSENSUS_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_watchdog_proof_consensus_evidence_contract");
}

const CHECKLIST_CONTAINS_GOVERNANCE_SIMULATION_AND_HUMAN_VETO_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Governance Simulation and Human-Veto Evidence Contract",
    "generate_governance_simulation_evidence_bundle.sh",
    "check_governance_simulation_policy.sh",
    "governance_simulation_contract_lane_contract.py",
    "framework.contract_lane_helpers",
    "run_governance_simulation_contract_lane.sh",
    "run_governance_simulation_deep_lane.sh",
    "run_governance_simulation_matrix.py",
    "fixtures/governance_simulation/veto_timelock_cases.json",
];

#[test]
fn checklist_contains_governance_simulation_and_human_veto_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_GOVERNANCE_SIMULATION_AND_HUMAN_VETO_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_governance_simulation_and_human_veto_evidence_contract");
}
