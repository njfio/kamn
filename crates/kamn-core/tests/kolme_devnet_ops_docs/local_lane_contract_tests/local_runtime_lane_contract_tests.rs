use super::super::docs_assert_support::assert_plan_contains_all;

const PLAN_CONTAINS_RUNTIME_COMMIT_ADAPTER_REPLAY_LANE_POLICY_PLAN_MARKERS: &[&str] = &[
    "## Runtime Commit Adapter Replay/Finality Fast Lane",
    "run_runtime_commit_adapter_contract_lane.sh",
    "receipt_provider_mismatch",
    "receipt_not_final",
];

#[test]
fn plan_contains_runtime_commit_adapter_replay_lane_policy() {
    assert_plan_contains_all(PLAN_CONTAINS_RUNTIME_COMMIT_ADAPTER_REPLAY_LANE_POLICY_PLAN_MARKERS, "plan_contains_runtime_commit_adapter_replay_lane_policy");
}

const PLAN_CONTAINS_RUNTIME_COMMIT_BLOCK_FALLBACK_FAST_LANE_POLICY_PLAN_MARKERS: &[&str] = &[
    "## Runtime Commit Block Fallback Reconciliation Fast Lane",
    "run_block_fallback_reconciliation_contract_lane.sh",
    "kolme_runtime_commit_block_fallback",
];

#[test]
fn plan_contains_runtime_commit_block_fallback_fast_lane_policy() {
    assert_plan_contains_all(PLAN_CONTAINS_RUNTIME_COMMIT_BLOCK_FALLBACK_FAST_LANE_POLICY_PLAN_MARKERS, "plan_contains_runtime_commit_block_fallback_fast_lane_policy");
}

const PLAN_CONTAINS_LOCAL_KOLME_API_PROBE_LANE_PLAN_MARKERS: &[&str] = &[
    "## Deterministic Local Kolme API Probe Lane",
    "run_local_kolme_api_probe_lane.sh",
    "--fork-chain-version v0.15.2",
    "GET /fork-info?chain_version=<version>",
    "kamn.kolme.local-api-probe-summary.v1",
];

#[test]
fn plan_contains_local_kolme_api_probe_lane() {
    assert_plan_contains_all(PLAN_CONTAINS_LOCAL_KOLME_API_PROBE_LANE_PLAN_MARKERS, "plan_contains_local_kolme_api_probe_lane");
}

const PLAN_CONTAINS_LOCAL_KOLME_API_SMOKE_LANE_PLAN_MARKERS: &[&str] = &[
    "## Bounded Local-Only Kolme API Smoke Lane",
    "run_local_kolme_api_smoke_lane.sh",
    "kamn.kolme.local-api-smoke-summary.v1",
];

#[test]
fn plan_contains_local_kolme_api_smoke_lane() {
    assert_plan_contains_all(PLAN_CONTAINS_LOCAL_KOLME_API_SMOKE_LANE_PLAN_MARKERS, "plan_contains_local_kolme_api_smoke_lane");
}

const PLAN_CONTAINS_LOCAL_LIVE_API_CONFORMANCE_HARNESS_PLAN_MARKERS: &[&str] = &[
    "## Local-Only Live Kolme API Conformance Harness",
    "run_local_kolme_live_api_conformance_harness.sh",
    "check_local_kolme_live_api_conformance_policy.py",
    "run_local_kolme_live_api_conformance_contract_lane.sh",
    "fixtures/kolme_commit/local_live_api_conformance_matrix.json",
    "kamn.kolme.local-live-api-conformance-summary.v1",
];

#[test]
fn plan_contains_local_live_api_conformance_harness() {
    assert_plan_contains_all(PLAN_CONTAINS_LOCAL_LIVE_API_CONFORMANCE_HARNESS_PLAN_MARKERS, "plan_contains_local_live_api_conformance_harness");
}

const PLAN_CONTAINS_LOCAL_KAMN_LIVE_RUNTIME_INTEGRATION_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local KAMN Live Runtime Integration Lane",
    "run_local_kamn_live_runtime_integration_lane.sh",
    "check_local_kamn_live_runtime_integration_policy.py",
    "run_local_kamn_live_runtime_integration_contract_lane.sh",
    "run_local_runtime_commit_live_finality_evidence_contract_lane.sh",
    "--runtime-commit-live-policy-report",
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY",
    "runtime_commit_managed_external_signer_public_key_marker_missing",
    "managed_signer_public_key_marker_missing",
    "managed_signer_public_key_marker_invalid",
    "run_localhost_signed_integration_contract_lane.sh",
    "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
    "composite_gate_reason_taxonomy_version=kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1",
    "composite_gate_reason_codes_csv=dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded",
    "composite_gate_evidence_convergence_status=verified",
    "composite_gate_ci_smoke_local_heavy_boundary_status=verified",
    "composite_gate_ci_smoke_lane_cost_profile=low",
    "composite_gate_local_heavy_execution_mode=not_requested",
    "signed runtime-commit envelope translation enforces `signer_key_id` presence and canonical message/signature binding before broadcast normalization.",
    "finality verification uses `/notifications` first with bounded `/block/{height}` fallback; no runtime commit status endpoint dependency.",
    "`Regression: #2101`",
];

#[test]
fn plan_contains_local_kamn_live_runtime_integration_lane() {
    assert_plan_contains_all(PLAN_CONTAINS_LOCAL_KAMN_LIVE_RUNTIME_INTEGRATION_LANE_PLAN_MARKERS, "plan_contains_local_kamn_live_runtime_integration_lane");
}

const PLAN_CONTAINS_UNIFIED_LOCAL_SIGNED_TO_KOLME_DEMO_LANE_PLAN_MARKERS: &[&str] = &[
    "## Unified Local Signed-to-Kolme Demo Contract Lane",
    "run_local_signed_to_kolme_demo_contract_lane.sh",
    "check_local_signed_to_kolme_demo_policy.py",
    "kamn.kolme.local-signed-to-kolme-demo-summary.v1",
    "runtime_signing_profile_contract_version=v1",
    "runtime_signing_profile=kolme-fork-secp256k1-v1",
    "native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1",
    "native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch",
    "Regression: #4373",
    "Regression: #4380",
];

#[test]
fn plan_contains_unified_local_signed_to_kolme_demo_lane() {
    assert_plan_contains_all(PLAN_CONTAINS_UNIFIED_LOCAL_SIGNED_TO_KOLME_DEMO_LANE_PLAN_MARKERS, "plan_contains_unified_local_signed_to_kolme_demo_lane");
}

const PLAN_CONTAINS_LOCAL_RUNTIME_COMMIT_LIVE_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Runtime Commit Live Proof Lane",
    "run_local_runtime_commit_live_lane.sh",
    "check_local_runtime_commit_live_evidence_policy.py",
    "run_local_runtime_commit_live_finality_evidence_contract_lane.sh",
    "submit_evidence_marker_present",
    "finality_evidence_marker_present",
    "request_payload_evidence_artifact_path_lineage_mismatch",
    "submit_evidence_artifact_path_lineage_mismatch",
    "finality_evidence_artifact_path_lineage_mismatch",
    "provider_failure_reason_taxonomy_version=kamn.kolme.local-runtime-commit-provider-failure-reason-taxonomy.v1",
    "provider_failure_reason_codes_csv=provider_client_contract_mismatch,provider_contract_enforcement_mode_mismatch,provider_live_contract_marker_mismatch,provider_live_contract_marker_missing,provider_in_memory_reference_detected,provider_hint_in_memory_provider_reference_detected,provider_submit_profile_contract_mismatch,provider_command_marker_mismatch,provider_command_marker_missing,provider_signing_profile_marker_mismatch,provider_signing_profile_marker_missing,provider_signing_profile_simulated_detected,provider_signer_adapter_contract_mismatch,provider_signing_curve_contract_mismatch,provider_signing_profile_contract_version_mismatch,live_command_in_memory_provider_reference_detected",
    "kolme.live.submit.retry",
    "kolme.live.finality.retry",
    "kolme.live.submit.retry.terminal",
    "kolme.live.finality.retry.terminal",
    "decision=retry",
    "jitter_seed",
    "terminal_decision=attempt_ceiling_reached",
    "terminal_decision=malformed_response_fail_fast",
    "submit_retry_terminal_decision",
    "finality_retry_terminal_decision",
    "retry_jitter_seed",
    "retry_tls_smoke_contract_status=verified",
    "retry_tls_live_https_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1",
    "retry_tls_submit_finality_taxonomy_version=kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1",
    "retry/tls local-heavy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
    "kamn.kolme.local-runtime-commit-live-summary.v1",
    "`Regression: #2099`",
    "`Regression: #4110`",
];

#[test]
fn plan_contains_local_runtime_commit_live_lane() {
    assert_plan_contains_all(PLAN_CONTAINS_LOCAL_RUNTIME_COMMIT_LIVE_LANE_PLAN_MARKERS, "plan_contains_local_runtime_commit_live_lane");
}

const PLAN_CONTAINS_LOCAL_NATIVE_API_PARITY_LIVE_PROOF_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Native API Parity Live Proof Lane",
    "run_local_native_api_parity_live_proof_lane.sh",
    "check_local_native_api_parity_live_proof_policy.py",
    "run_local_native_api_parity_live_proof_contract_lane.sh",
    "kamn.kolme.local-native-api-parity-live-proof-summary.v1",
];

#[test]
fn plan_contains_local_native_api_parity_live_proof_lane() {
    assert_plan_contains_all(PLAN_CONTAINS_LOCAL_NATIVE_API_PARITY_LIVE_PROOF_LANE_PLAN_MARKERS, "plan_contains_local_native_api_parity_live_proof_lane");
}
