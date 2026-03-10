use super::support::assert_doc_contains_all;

const LOCAL_KOLME_LANE_MARKERS: &[&str] = &[
    "run_local_fork_sync_metadata_lane.sh --mode run",
    "run_local_fork_smoke_evidence_lane.sh --mode run",
    "run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2",
    "run_local_kolme_api_smoke_lane.sh --mode run",
    "run_local_kolme_live_api_conformance_harness.sh --mode run",
    "check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json --phase contract --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json",
    "run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run",
    "check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json --phase contract --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json",
    "run_local_kamn_live_runtime_integration_lane.sh --mode run",
    "check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json --phase contract --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json",
    "composite_gate_reason_taxonomy_version=kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1",
    "composite_gate_reason_codes_csv=dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded",
    "composite_gate_evidence_convergence_status=verified",
    "composite_gate_ci_smoke_local_heavy_boundary_status=verified",
    "composite_gate_ci_smoke_lane_cost_profile=low",
    "composite_gate_local_heavy_execution_mode=not_requested",
    "run_local_kolme_fork_process_lifecycle_lane.sh --mode run",
    "check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json --phase contract --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json",
    "run_local_kolme_fork_profile_preflight_contract_lane.sh",
    "test_run_local_kolme_fork_profile_preflight_contract_lane.sh",
    "run_local_kolme_fork_self_test_contract_lane.sh",
    "test_run_local_kolme_fork_self_test_contract_lane.sh",
    "run_local_kolme_fork_portability_preflight_contract_lane.sh",
    "test_run_local_kolme_fork_portability_preflight_contract_lane.sh",
    "run_local_runtime_commit_live_lane.sh --mode run",
    "run_local_native_api_parity_live_proof_lane.sh --mode run",
    "--request PUT --data '{\\\"message\\\":\\\"native-parity\\\",\\\"signature\\\":\\\"sig\\\",\\\"recovery_id\\\":1}' http://127.0.0.1:3000/broadcast",
];

const LOCAL_KOLME_CONTRACT_BOUNDARY_MARKERS: &[&str] = &[
    "test_run_local_runtime_commit_live_lane.sh",
    "test_run_local_native_api_parity_live_proof_contract_lane.sh",
    "test_run_local_kolme_live_api_conformance_contract_lane.sh",
    "test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh",
    "test_run_local_kamn_live_runtime_integration_contract_lane.sh",
    "test_run_local_kolme_fork_process_lifecycle_contract_lane.sh",
    "run_nonce_broadcast_parity_contract_lane.sh",
    "test_run_nonce_broadcast_parity_contract_lane.sh",
    "KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60",
    "run_local_bootstrap_health_checks.sh",
    "check_local_bootstrap_health_policy.py",
    "run_local_bootstrap_health_checks_contract_lane.sh",
    "test_check_local_bootstrap_health_policy.sh",
    "test_run_local_bootstrap_health_checks_contract_lane.sh",
    "run_local_e2e_integration_lane.sh",
    "check_local_e2e_integration_policy.py",
    "run_local_e2e_integration_contract_lane.sh",
    "run_local_heavy_validation_matrix.sh",
    "check_local_heavy_validation_matrix_policy.py",
    "run_local_heavy_validation_matrix_contract_lane.sh",
    "KAMN_KOLME_LOCAL_HEAVY=1",
    "local-only heavy Kolme run-mode commands remain excluded from ci-fast-gate.",
    "kolme_local_heavy_lane_mode=local-only|manual-opt-in|not-applicable",
    "manual-hardened mode: manual",
];

#[test]
fn doc_contains_local_kolme_lane_markers() {
    assert_doc_contains_all(LOCAL_KOLME_LANE_MARKERS, "local kolme lane");
}

#[test]
fn doc_contains_local_kolme_boundary_markers() {
    assert_doc_contains_all(
        LOCAL_KOLME_CONTRACT_BOUNDARY_MARKERS,
        "local kolme boundary",
    );
}
