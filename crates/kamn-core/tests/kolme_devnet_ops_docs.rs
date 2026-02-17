const PLAN: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");

#[test]
fn plan_contains_triadic_smoke_contract_commands() {
    assert!(PLAN.contains("run_triadic_devnet_smoke.sh"));
    assert!(PLAN.contains("validate_triadic_devnet_smoke.py"));
    assert!(PLAN.contains("run_triadic_devnet_smoke_contract_lane.sh"));
}

#[test]
fn plan_contains_fallback_marker_retirement_matrix_contract() {
    assert!(PLAN.contains("## Fallback Marker Retirement Matrix (Issue #2526)"));
    assert!(PLAN.contains("fixtures/kolme_compatibility/fallback_signer_marker_matrix.json"));
    assert!(PLAN.contains("kamn.kolme.fallback-signer-marker-matrix.v1"));
    assert!(PLAN.contains("check_fallback_signer_marker_matrix_policy.py"));
    assert!(PLAN.contains("bash scripts/kolme/test_check_fallback_signer_marker_matrix_policy.sh"));
    assert!(PLAN.contains("remove-target"));
}

#[test]
fn plan_contains_failover_sync_drill_lane_policy() {
    assert!(PLAN.contains("## Failover + Sync Drill Lane Policy"));
    assert!(PLAN.contains("select_failover_sync_drill_lane.sh"));
    assert!(PLAN.contains("run_failover_sync_drill_preflight_contract_lane.sh"));
    assert!(PLAN.contains("run_failover_sync_drill_deep_lane.sh"));
    assert!(PLAN.contains("run_failover_sync_drill_suite.sh"));
    assert!(PLAN.contains("kamn.runtime.failover-sync-drill-suite-report.v1"));
}

#[test]
fn plan_contains_runtime_commit_adapter_replay_lane_policy() {
    assert!(PLAN.contains("## Runtime Commit Adapter Replay/Finality Fast Lane"));
    assert!(PLAN.contains("run_runtime_commit_adapter_contract_lane.sh"));
    assert!(PLAN.contains("receipt_provider_mismatch"));
    assert!(PLAN.contains("receipt_not_final"));
}

#[test]
fn plan_contains_runtime_commit_block_fallback_fast_lane_policy() {
    assert!(PLAN.contains("## Runtime Commit Block Fallback Reconciliation Fast Lane"));
    assert!(PLAN.contains("run_block_fallback_reconciliation_contract_lane.sh"));
    assert!(PLAN.contains("kolme_runtime_commit_block_fallback"));
}

#[test]
fn plan_contains_local_fork_sync_metadata_lane() {
    assert!(PLAN.contains("## Deterministic Local Fork Sync Metadata Lane"));
    assert!(PLAN.contains("run_local_fork_sync_metadata_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-sync-metadata-summary.v1"));
    assert!(PLAN.contains("expected-remote-url https://github.com/njfio/kolme_fork.git"));
}

#[test]
fn plan_contains_local_fork_smoke_evidence_lane() {
    assert!(PLAN.contains("## Bounded Local Fork Smoke Evidence Lane"));
    assert!(PLAN.contains("run_local_fork_smoke_evidence_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-smoke-evidence-summary.v1"));
    assert!(PLAN.contains("fork_smoke_command_timeout"));
}

#[test]
fn plan_contains_local_kolme_api_probe_lane() {
    assert!(PLAN.contains("## Deterministic Local Kolme API Probe Lane"));
    assert!(PLAN.contains("run_local_kolme_api_probe_lane.sh"));
    assert!(PLAN.contains("--fork-chain-version v0.15.2"));
    assert!(PLAN.contains("GET /fork-info?chain_version=<version>"));
    assert!(PLAN.contains("kamn.kolme.local-api-probe-summary.v1"));
}

#[test]
fn plan_contains_local_kolme_api_smoke_lane() {
    assert!(PLAN.contains("## Bounded Local-Only Kolme API Smoke Lane"));
    assert!(PLAN.contains("run_local_kolme_api_smoke_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-api-smoke-summary.v1"));
}

#[test]
fn plan_contains_local_live_api_conformance_harness() {
    assert!(PLAN.contains("## Local-Only Live Kolme API Conformance Harness"));
    assert!(PLAN.contains("run_local_kolme_live_api_conformance_harness.sh"));
    assert!(PLAN.contains("check_local_kolme_live_api_conformance_policy.py"));
    assert!(PLAN.contains("run_local_kolme_live_api_conformance_contract_lane.sh"));
    assert!(PLAN.contains("fixtures/kolme_commit/local_live_api_conformance_matrix.json"));
    assert!(PLAN.contains("kamn.kolme.local-live-api-conformance-summary.v1"));
}

#[test]
fn plan_contains_staged_rehearsal_signoff_artifact_contract() {
    assert!(PLAN.contains("## Staged Rehearsal Signoff Artifact Contract (Issue #3241)"));
    assert!(PLAN.contains("run_staging_rehearsal_contract_lane.sh"));
    assert!(PLAN.contains("check_staging_rehearsal_policy.sh"));
    assert!(PLAN.contains("kamn.release.staged-rehearsal-signoff.v1"));
    assert!(PLAN.contains("staged_rehearsal_signoff_status=verified|fail-closed"));
}

#[test]
fn plan_contains_local_fork_bootstrap_readiness_contract_lane() {
    assert!(PLAN.contains("## Local Kolme Fork Bootstrap/Readiness Contract Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_bootstrap_readiness_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_bootstrap_readiness_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_bootstrap_readiness_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-bootstrap-readiness-summary.v1"));
}

#[test]
fn plan_contains_local_kamn_live_runtime_integration_lane() {
    assert!(PLAN.contains("## Local KAMN Live Runtime Integration Lane"));
    assert!(PLAN.contains("run_local_kamn_live_runtime_integration_lane.sh"));
    assert!(PLAN.contains("check_local_kamn_live_runtime_integration_policy.py"));
    assert!(PLAN.contains("run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(PLAN.contains("run_local_runtime_commit_live_finality_evidence_contract_lane.sh"));
    assert!(PLAN.contains("--runtime-commit-live-policy-report"));
    assert!(PLAN.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"));
    assert!(PLAN.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"));
    assert!(PLAN.contains("runtime_commit_managed_external_signer_public_key_marker_missing"));
    assert!(PLAN.contains("managed_signer_public_key_marker_missing"));
    assert!(PLAN.contains("managed_signer_public_key_marker_invalid"));
    assert!(PLAN.contains("run_localhost_signed_integration_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-kamn-live-runtime-integration-summary.v1"));
    assert!(PLAN.contains(
        "composite_gate_reason_taxonomy_version=kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1"
    ));
    assert!(PLAN.contains(
        "composite_gate_reason_codes_csv=dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded"
    ));
    assert!(PLAN.contains("composite_gate_evidence_convergence_status=verified"));
    assert!(PLAN.contains("composite_gate_ci_smoke_local_heavy_boundary_status=verified"));
    assert!(PLAN.contains("composite_gate_ci_smoke_lane_cost_profile=low"));
    assert!(PLAN.contains("composite_gate_local_heavy_execution_mode=not_requested"));
    assert!(PLAN.contains(
        "signed runtime-commit envelope translation enforces `signer_key_id` presence and canonical message/signature binding before broadcast normalization."
    ));
    assert!(PLAN.contains(
        "finality verification uses `/notifications` first with bounded `/block/{height}` fallback; no runtime commit status endpoint dependency."
    ));
    assert!(PLAN.contains("`Regression: #2101`"));
}

#[test]
fn plan_contains_unified_local_signed_to_kolme_demo_lane() {
    assert!(PLAN.contains("## Unified Local Signed-to-Kolme Demo Contract Lane"));
    assert!(PLAN.contains("run_local_signed_to_kolme_demo_contract_lane.sh"));
    assert!(PLAN.contains("check_local_signed_to_kolme_demo_policy.py"));
    assert!(PLAN.contains("kamn.kolme.local-signed-to-kolme-demo-summary.v1"));
    assert!(PLAN.contains("runtime_signing_profile_contract_version=v1"));
    assert!(PLAN.contains("runtime_signing_profile=kolme-fork-secp256k1-v1"));
    assert!(PLAN.contains(
        "native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1"
    ));
    assert!(PLAN.contains(
        "native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch"
    ));
    assert!(PLAN.contains("Regression: #4373"));
    assert!(PLAN.contains("Regression: #4380"));
}

#[test]
fn plan_contains_local_fork_process_lifecycle_lane() {
    assert!(PLAN.contains("## Local Kolme Fork Process Lifecycle Integration Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_process_lifecycle_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_process_lifecycle_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_process_lifecycle_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-process-lifecycle-summary.v1"));
}

#[test]
fn plan_contains_local_fork_profile_preflight_lane() {
    assert!(PLAN.contains("## Local Fork Profile Preflight Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_profile_preflight_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_profile_preflight_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_profile_preflight_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-profile-preflight-summary.v1"));
    assert!(PLAN.contains("kamn.kolme.local-fork-profile-preflight-policy-report.v1"));
}

#[test]
fn plan_contains_local_fork_self_test_lane() {
    assert!(PLAN.contains("## Local Fork Self-Test Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_self_test_lane.sh"));
    assert!(PLAN.contains("--matrix-cargo-profile portable"));
    assert!(PLAN.contains("check_local_kolme_fork_self_test_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_self_test_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-self-test-summary.v1"));
    assert!(PLAN.contains("kamn.kolme.local-fork-self-test-policy-report.v1"));
}

#[test]
fn plan_contains_local_fork_portability_preflight_lane() {
    assert!(PLAN.contains("## Local Fork Portability Preflight Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_portability_preflight_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_portability_preflight_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_portability_preflight_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-portability-preflight-summary.v1"));
    assert!(PLAN.contains("kamn.kolme.local-fork-portability-preflight-policy-report.v1"));
}

#[test]
fn plan_contains_local_fork_checkout_bootstrap_lane() {
    assert!(PLAN.contains("## Local Fork Checkout Bootstrap Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_checkout_bootstrap_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_checkout_bootstrap_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_checkout_bootstrap_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-checkout-bootstrap-summary.v1"));
}

#[test]
fn plan_contains_real_fork_local_process_wrapper_lane() {
    assert!(PLAN.contains("## Real Fork Local Process Wrapper Contract Lane"));
    assert!(PLAN.contains("run_local_kolme_fork_real_process_contract_lane.sh"));
    assert!(PLAN.contains("run_local_kolme_fork_checkout_bootstrap_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_checkout_bootstrap_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_profile_preflight_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_profile_preflight_policy.py"));
    assert!(PLAN.contains("run_local_kolme_fork_self_test_lane.sh"));
    assert!(PLAN.contains("check_local_kolme_fork_self_test_policy.py"));
    assert!(PLAN.contains("check_local_kolme_fork_real_process_policy.py"));
    assert!(PLAN.contains("kamn.kolme.local-fork-real-process-summary.v1"));
}

#[test]
fn plan_contains_real_fork_wrapper_policy_checker_test_command() {
    assert!(PLAN.contains("test_check_local_kolme_fork_real_process_policy.sh"));
}

#[test]
fn plan_contains_local_runtime_commit_live_lane() {
    assert!(PLAN.contains("## Local Runtime Commit Live Proof Lane"));
    assert!(PLAN.contains("run_local_runtime_commit_live_lane.sh"));
    assert!(PLAN.contains("check_local_runtime_commit_live_evidence_policy.py"));
    assert!(PLAN.contains("run_local_runtime_commit_live_finality_evidence_contract_lane.sh"));
    assert!(PLAN.contains("submit_evidence_marker_present"));
    assert!(PLAN.contains("finality_evidence_marker_present"));
    assert!(PLAN.contains("request_payload_evidence_artifact_path_lineage_mismatch"));
    assert!(PLAN.contains("submit_evidence_artifact_path_lineage_mismatch"));
    assert!(PLAN.contains("finality_evidence_artifact_path_lineage_mismatch"));
    assert!(PLAN.contains(
        "provider_failure_reason_taxonomy_version=kamn.kolme.local-runtime-commit-provider-failure-reason-taxonomy.v1"
    ));
    assert!(PLAN.contains(
        "provider_failure_reason_codes_csv=provider_client_contract_mismatch,provider_contract_enforcement_mode_mismatch,provider_live_contract_marker_mismatch,provider_live_contract_marker_missing,provider_in_memory_reference_detected,provider_hint_in_memory_provider_reference_detected,provider_submit_profile_contract_mismatch,provider_command_marker_mismatch,provider_command_marker_missing,provider_signing_profile_marker_mismatch,provider_signing_profile_marker_missing,provider_signing_profile_simulated_detected,provider_signer_adapter_contract_mismatch,provider_signing_curve_contract_mismatch,provider_signing_profile_contract_version_mismatch,live_command_in_memory_provider_reference_detected"
    ));
    assert!(PLAN.contains("kolme.live.submit.retry"));
    assert!(PLAN.contains("kolme.live.finality.retry"));
    assert!(PLAN.contains("kolme.live.submit.retry.terminal"));
    assert!(PLAN.contains("kolme.live.finality.retry.terminal"));
    assert!(PLAN.contains("decision=retry"));
    assert!(PLAN.contains("jitter_seed"));
    assert!(PLAN.contains("terminal_decision=attempt_ceiling_reached"));
    assert!(PLAN.contains("terminal_decision=malformed_response_fail_fast"));
    assert!(PLAN.contains("submit_retry_terminal_decision"));
    assert!(PLAN.contains("finality_retry_terminal_decision"));
    assert!(PLAN.contains("retry_jitter_seed"));
    assert!(PLAN.contains("retry_tls_smoke_contract_status=verified"));
    assert!(PLAN.contains(
        "retry_tls_live_https_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1"
    ));
    assert!(PLAN.contains(
        "retry_tls_submit_finality_taxonomy_version=kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1"
    ));
    assert!(
        PLAN.contains(
            "retry/tls local-heavy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
        )
    );
    assert!(PLAN.contains("kamn.kolme.local-runtime-commit-live-summary.v1"));
    assert!(PLAN.contains("`Regression: #2099`"));
    assert!(PLAN.contains("`Regression: #4110`"));
}

#[test]
fn plan_contains_local_native_api_parity_live_proof_lane() {
    assert!(PLAN.contains("## Local Native API Parity Live Proof Lane"));
    assert!(PLAN.contains("run_local_native_api_parity_live_proof_lane.sh"));
    assert!(
        PLAN.contains("--request PUT --data '{\\\"message\\\":\\\"native-parity\\\",\\\"signature\\\":\\\"sig\\\",\\\"recovery_id\\\":1}' http://127.0.0.1:3000/broadcast")
    );
    assert!(PLAN.contains("check_local_native_api_parity_live_proof_policy.py"));
    assert!(PLAN.contains("run_local_native_api_parity_live_proof_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-native-api-parity-live-proof-summary.v1"));
}

#[test]
fn plan_contains_fast_gate_native_api_parity_lane() {
    assert!(PLAN.contains("## Fast-Gate Native API Parity Contract Lane"));
    assert!(PLAN.contains("run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(PLAN.contains("check_fast_gate_native_api_parity_policy.py"));
    assert!(PLAN.contains("kamn.kolme.fast-gate-native-api-parity-summary.v1"));
    assert!(PLAN.contains("KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS"));
    assert!(PLAN.contains("test_run_fast_gate_native_api_parity_contract_lane.sh"));
}

#[test]
fn plan_contains_local_only_heavy_validation_matrix() {
    assert!(PLAN.contains("## Local-Only Heavy Kolme Validation Matrix"));
    assert!(PLAN.contains("run_local_heavy_validation_matrix.sh"));
    assert!(PLAN.contains("check_local_heavy_validation_matrix_policy.py"));
    assert!(PLAN.contains("run_local_heavy_validation_matrix_contract_lane.sh"));
    assert!(PLAN.contains("--cargo-profile portable"));
    assert!(PLAN.contains("run_local_bootstrap_health_checks.sh"));
    assert!(PLAN.contains("run_version_compatibility_replay_deep_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-heavy-validation-summary.v1"));
    assert!(PLAN.contains("kamn.kolme.local-heavy-validation-policy-report.v1"));
}

#[test]
fn plan_contains_deterministic_local_bootstrap_health_checks() {
    assert!(PLAN.contains("## Deterministic Local Bootstrap Health Checks"));
    assert!(PLAN.contains("run_local_bootstrap_health_checks.sh"));
    assert!(PLAN.contains("check_local_bootstrap_health_policy.py"));
    assert!(PLAN.contains("run_local_bootstrap_health_checks_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-bootstrap-summary.v1"));
    assert!(PLAN.contains("kamn.kolme.local-bootstrap-policy-report.v1"));
    assert!(PLAN.contains("KAMN_KOLME_LOCAL_HEAVY=1"));
}

#[test]
fn plan_contains_local_only_heavy_e2e_lane() {
    assert!(PLAN.contains("## Local-Only Heavy End-to-End Lane"));
    assert!(PLAN.contains("run_local_e2e_integration_lane.sh"));
    assert!(PLAN.contains("check_local_e2e_integration_policy.py"));
    assert!(PLAN.contains("run_local_e2e_integration_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-e2e-integration-summary.v1"));
    assert!(PLAN.contains("kamn.kolme.local-e2e-integration-policy-report.v1"));
    assert!(PLAN.contains("run_runtime_commit_adapter_contract_lane.sh"));
    assert!(PLAN.contains("run_live_transport_parity_contract_lane.sh"));
}

#[test]
fn plan_contains_lane_migration_matrix_contract() {
    assert!(PLAN.contains("## Lane Migration Matrix (Issue #1721)"));
    assert!(PLAN.contains("fixtures/kolme_compatibility/lane_migration_matrix.json"));
    assert!(PLAN.contains("kamn.kolme.lane-migration-matrix.v1"));
    assert!(PLAN.contains("check_lane_migration_matrix_policy.py"));
    assert!(PLAN.contains("test_check_lane_migration_matrix_policy.sh"));
    assert!(PLAN.contains("kolme.local.fork.rust_matrix"));
}

#[test]
fn plan_contains_tranche1_manifest_migration_contract() {
    assert!(PLAN.contains("## Tranche-1 Manifest Migration (Issue #1722)"));
    assert!(PLAN.contains("scripts/ci/test_kolme_tranche1_manifest_migration_contract.sh"));
    assert!(PLAN.contains("scripts/framework/manifests/kolme_snapshot_drift_contract_lane.json"));
    assert!(PLAN
        .contains("scripts/framework/manifests/kolme_notifications_consumer_contract_lane.json"));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_block_fallback_reconciliation_contract_lane.json"
    ));
    assert!(PLAN.contains("scripts/kolme/contracts/block_fallback_reconciliation_contract_lane.py"));
    assert!(PLAN.contains("Combined wrapper shell LOC for the tranche remains `<= 60`."));
}

#[test]
fn plan_contains_runtime_nonce_manifest_migration_contract() {
    assert!(PLAN.contains("## Runtime+Nonce Manifest Migration (Issue #1763)"));
    assert!(PLAN.contains("scripts/ci/test_kolme_runtime_nonce_manifest_migration_contract.sh"));
    assert!(PLAN
        .contains("scripts/framework/manifests/kolme_runtime_commit_adapter_contract_lane.json"));
    assert!(
        PLAN.contains("scripts/framework/manifests/kolme_runtime_commit_replay_contract_lane.json")
    );
    assert!(PLAN
        .contains("scripts/framework/manifests/kolme_nonce_broadcast_parity_contract_lane.json"));
    assert!(PLAN.contains("scripts/kolme/contracts/runtime_commit_replay_contract_lane.py"));
    assert!(PLAN
        .contains("Combined wrapper shell LOC for this runtime/nonce tranche remains `<= 120`."));
}

#[test]
fn plan_contains_version_matrix_manifest_migration_contract() {
    assert!(PLAN.contains("## Version+Matrix Manifest Migration (Issue #1765)"));
    assert!(PLAN.contains("scripts/ci/test_kolme_version_matrix_manifest_migration_contract.sh"));
    assert!(
        PLAN.contains("scripts/framework/manifests/kolme_version_compatibility_contract_lane.json")
    );
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_fork_rust_test_matrix_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_heavy_validation_matrix_contract_lane.json"
    ));
    assert!(PLAN.contains("scripts/kolme/contracts/local_heavy_validation_matrix_contract_lane.py"));
    assert!(PLAN
        .contains("Combined wrapper shell LOC for this version/matrix tranche remains `<= 120`."));
}

#[test]
fn plan_contains_profile_selftest_portability_manifest_migration_contract() {
    assert!(PLAN.contains("## Profile+SelfTest+Portability Manifest Migration (Issue #1767)"));
    assert!(PLAN.contains(
        "scripts/ci/test_kolme_profile_selftest_portability_manifest_migration_contract.sh"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_fork_profile_preflight_contract_lane.json"
    ));
    assert!(
        PLAN.contains("scripts/framework/manifests/kolme_local_fork_self_test_contract_lane.json")
    );
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_fork_portability_preflight_contract_lane.json"
    ));
    assert!(PLAN.contains("scripts/kolme/contracts/local_fork_self_test_contract_lane.py"));
    assert!(PLAN.contains(
        "Combined wrapper shell LOC for this profile/self-test/portability tranche remains `<= 120`."
    ));
}

#[test]
fn plan_contains_runtime_triadic_bootstrap_e2e_manifest_migration_contract() {
    assert!(PLAN.contains("## Runtime+Triadic+Bootstrap+E2E Manifest Migration (Issue #1769)"));
    assert!(PLAN.contains(
        "scripts/ci/test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh"
    ));
    assert!(PLAN.contains("scripts/framework/manifests/kolme_runtime_commit_contract_lane.json"));
    assert!(
        PLAN.contains("scripts/framework/manifests/kolme_triadic_devnet_smoke_contract_lane.json")
    );
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_bootstrap_health_checks_contract_lane.json"
    ));
    assert!(
        PLAN.contains("scripts/framework/manifests/kolme_local_e2e_integration_contract_lane.json")
    );
    assert!(PLAN.contains("scripts/kolme/contracts/local_e2e_integration_contract_lane.py"));
    assert!(PLAN.contains(
        "Combined wrapper shell LOC for this runtime/triadic/bootstrap/e2e tranche remains `<= 160`."
    ));
}

#[test]
fn plan_contains_bootstrap_conformance_runtime_process_manifest_migration_contract() {
    assert!(
        PLAN.contains("## Bootstrap+Conformance+Runtime+Process Manifest Migration (Issue #1771)")
    );
    assert!(PLAN.contains(
        "scripts/ci/test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json"
    ));
    assert!(PLAN
        .contains("scripts/kolme/contracts/local_kolme_fork_process_lifecycle_contract_lane.py"));
    assert!(PLAN.contains(
        "Combined wrapper shell LOC for this bootstrap/conformance/runtime/process tranche remains `<= 160`."
    ));
}

#[test]
fn plan_contains_parity_demo_real_process_manifest_migration_contract() {
    assert!(PLAN.contains("## Parity+Demo+Real-Process Manifest Migration (Issue #1773)"));
    assert!(PLAN
        .contains("scripts/ci/test_kolme_parity_demo_real_process_manifest_migration_contract.sh"));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_fast_gate_native_api_parity_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_native_api_parity_live_proof_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_signed_to_kolme_demo_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_kolme_fork_checkout_bootstrap_contract_lane.json"
    ));
    assert!(PLAN.contains(
        "scripts/framework/manifests/kolme_local_kolme_fork_real_process_contract_lane.json"
    ));
    assert!(PLAN.contains("scripts/kolme/contracts/local_kolme_fork_real_process_contract_lane.py"));
    assert!(PLAN.contains(
        "Combined wrapper shell LOC for this parity/demo/real-process tranche remains `<= 200`."
    ));
}

#[test]
fn regression_requires_failover_sync_budget_and_scheduled_cadence_guards() {
    // Regression: #788
    assert!(PLAN
        .contains("Failover/sync budget overruns and unscheduled deep-lane execution fail closed"));
}

#[test]
fn regression_requires_runtime_commit_adapter_reason_code_guard() {
    // Regression: #980
    assert!(PLAN.contains(
        "runtime commit adapter replay/finality reason-code drift fails closed (`Regression: #980`)."
    ));
}

#[test]
fn regression_requires_local_only_heavy_matrix_guard_marker() {
    // Regression: #1405
    assert!(PLAN.contains(
        "local-only heavy validation matrix requires explicit opt-in and remains excluded from PR fast-gate workflows (`Regression: #1405`)."
    ));
}

#[test]
fn regression_requires_local_only_heavy_matrix_policy_contract_guard_marker() {
    // Regression: #1687
    assert!(PLAN.contains(
        "local-only heavy validation matrix summary policy and contract-lane command/report drift remain fail-closed (`Regression: #1687`)."
    ));
}

#[test]
fn regression_requires_local_bootstrap_policy_contract_guard_marker() {
    // Regression: #1692
    assert!(PLAN.contains(
        "local bootstrap health summary policy and contract-lane command/report drift remain fail-closed (`Regression: #1692`)."
    ));
}

#[test]
fn regression_requires_lane_migration_matrix_policy_guard_marker() {
    // Regression: #1721
    assert!(PLAN.contains(
        "lane migration matrix schema/order/required-lane drift remains fail-closed (`Regression: #1721`)."
    ));
}

#[test]
fn regression_requires_tranche1_manifest_migration_guard_marker() {
    // Regression: #1722
    assert!(PLAN.contains(
        "tranche-1 manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1722`)."
    ));
}

#[test]
fn regression_requires_runtime_nonce_manifest_migration_guard_marker() {
    // Regression: #1763
    assert!(PLAN.contains(
        "runtime+nonce manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1763`)."
    ));
}

#[test]
fn regression_requires_version_matrix_manifest_migration_guard_marker() {
    // Regression: #1765
    assert!(PLAN.contains(
        "version+matrix manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1765`)."
    ));
}

#[test]
fn regression_requires_profile_selftest_portability_manifest_migration_guard_marker() {
    // Regression: #1767
    assert!(PLAN.contains(
        "profile+self-test+portability manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1767`)."
    ));
}

#[test]
fn regression_requires_runtime_triadic_bootstrap_e2e_manifest_migration_guard_marker() {
    // Regression: #1769
    assert!(PLAN.contains(
        "runtime+triadic+bootstrap+e2e manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1769`)."
    ));
}

#[test]
fn regression_requires_bootstrap_conformance_runtime_process_manifest_migration_guard_marker() {
    // Regression: #1771
    assert!(PLAN.contains(
        "bootstrap+conformance+runtime+process manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1771`)."
    ));
}

#[test]
fn regression_requires_parity_demo_real_process_manifest_migration_guard_marker() {
    // Regression: #1773
    assert!(PLAN.contains(
        "parity+demo+real-process manifest migration wrapper routing and shell-LOC budget drift remains fail-closed (`Regression: #1773`)."
    ));
}

#[test]
fn regression_requires_local_bootstrap_opt_in_guard_marker() {
    // Regression: #1417
    assert!(PLAN.contains(
        "deterministic bootstrap run mode fails closed without explicit local-only opt-in (`Regression: #1417`)."
    ));
}

#[test]
fn regression_requires_local_e2e_opt_in_guard_marker() {
    // Regression: #1418
    assert!(PLAN.contains(
        "local-only heavy E2E lane run mode fails closed without explicit local-only opt-in (`Regression: #1418`)."
    ));
}

#[test]
fn regression_requires_local_e2e_policy_contract_guard_marker() {
    // Regression: #1682
    assert!(PLAN.contains(
        "local-only heavy E2E lane summary policy and contract-lane decision/checkpoint drift remain fail-closed (`Regression: #1682`)."
    ));
}

#[test]
fn regression_requires_shared_local_heavy_opt_in_helper_guard_marker() {
    // Regression: #1585
    assert!(PLAN.contains(
        "shared local-heavy opt-in helper wiring remains fail-closed across bootstrap/E2E/matrix lanes (`Regression: #1585`)."
    ));
}

#[test]
fn regression_requires_local_fork_sync_metadata_guard_marker() {
    // Regression: #1429
    assert!(PLAN.contains(
        "local fork metadata sync lane fails closed for checkout-path, remote-URL, ref, and dirty-checkout drift (`Regression: #1429`)."
    ));
}

#[test]
fn regression_requires_local_fork_smoke_evidence_guard_marker() {
    // Regression: #1430
    assert!(PLAN.contains(
        "local fork smoke evidence lane fails closed on missing local opt-in, metadata sync failure, command timeout, and smoke-command errors (`Regression: #1430`)."
    ));
}

#[test]
fn regression_requires_local_fork_checkout_bootstrap_guard_marker() {
    // Regression: #1663
    assert!(PLAN.contains(
        "local fork checkout bootstrap lane fails closed for local opt-in, checkout provenance drift, diagnostics command failures, and runtime budget overruns (`Regression: #1663`)."
    ));
}

#[test]
fn regression_requires_local_fork_matrix_portable_cargo_profile_guard_marker() {
    // Regression: #1659
    assert!(PLAN.contains(
        "local fork Rust test matrix portable cargo profile (`--cargo-profile portable`) remains fail-closed and linker-portable via `RUSTFLAGS=''` cargo override (`Regression: #1659`)."
    ));
}

#[test]
fn regression_requires_local_kolme_api_probe_guard_marker() {
    // Regression: #1439
    assert!(PLAN.contains(
        "local Kolme API probe lane fails closed on unavailable health endpoint, invalid fork-info payload, and runtime budget overruns (`Regression: #1439`)."
    ));
}

#[test]
fn regression_requires_local_kolme_api_smoke_guard_marker() {
    // Regression: #1440
    assert!(PLAN.contains(
        "local Kolme API smoke lane fails closed without explicit local opt-in, probe prerequisite failure, smoke-command timeout, and smoke-command errors (`Regression: #1440`)."
    ));
}

#[test]
fn regression_requires_local_runtime_commit_live_guard_marker() {
    // Regression: #1450
    assert!(PLAN.contains(
        "local runtime-commit live proof lane fails closed without local opt-in and for command timeout/failure paths (`Regression: #1450`)."
    ));
    assert!(PLAN.contains(
        "local runtime-commit live proof lane evidence policy remains fail-closed for missing live-provider command marker contracts (`Regression: #2095`)."
    ));
}

#[test]
fn regression_requires_runtime_commit_block_fallback_guard_marker() {
    // Regression: #1464
    assert!(PLAN.contains(
        "block fallback stale-window and response-height drift remains fail-closed (`Regression: #1464`)."
    ));
}

#[test]
fn regression_requires_local_native_api_parity_live_proof_guard_marker() {
    // Regression: #1465
    assert!(PLAN.contains(
        "local native API parity live proof lane fails closed without local opt-in and on nonce/broadcast/finality timeout or command failures (`Regression: #1465`)."
    ));
}

#[test]
fn regression_requires_native_parity_docs_matrix_guard_marker() {
    // Regression: #1468
    assert!(PLAN.contains(
        "native parity fast/local command matrix docs drift remains fail-closed (`Regression: #1468`)."
    ));
}

#[test]
fn regression_requires_live_kolme_method_and_query_guard_marker() {
    // Regression: #1482
    assert!(PLAN.contains(
        "local probe fork-info query semantics and native parity broadcast method drift remain fail-closed (`Regression: #1482`)."
    ));
}

#[test]
fn regression_requires_local_live_api_conformance_harness_guard_marker() {
    // Regression: #1483
    assert!(PLAN.contains(
        "local live API conformance harness fails closed for probe/native parity prerequisite failures, runtime budget overruns, and endpoint contract drift (`Regression: #1483`)."
    ));
}

#[test]
fn regression_requires_local_fork_bootstrap_readiness_guard_marker() {
    // Regression: #1488
    assert!(PLAN.contains(
        "local fork bootstrap/readiness lane fails closed for sync/probe prerequisite failures, runtime budget overruns, and missing local opt-in (`Regression: #1488`)."
    ));
}

#[test]
fn regression_requires_local_kamn_live_runtime_integration_guard_marker() {
    // Regression: #1489
    assert!(PLAN.contains(
        "local KAMN live runtime integration lane fails closed for bootstrap/localhost-signed/conformance/runtime-commit prerequisite drift, runtime budget overruns, and missing local opt-in (`Regression: #1489`)."
    ));
}

#[test]
fn regression_requires_localhost_signed_runtime_integration_prerequisite_guard_marker() {
    // Regression: #1636
    assert!(PLAN.contains(
        "local KAMN live runtime integration lane requires bounded localhost signed integration prerequisite execution before runtime commit submission (`Regression: #1636`)."
    ));
}

#[test]
fn regression_requires_unified_local_signed_to_kolme_demo_guard_marker() {
    // Regression: #1640
    assert!(PLAN.contains(
        "unified local signed-to-Kolme demo lane fails closed for local opt-in, stage prerequisite drift, and runtime budget overruns (`Regression: #1640`)."
    ));
}

#[test]
fn regression_requires_local_fork_process_lifecycle_guard_marker() {
    // Regression: #1494
    assert!(PLAN.contains(
        "local fork process lifecycle integration lane fails closed for process start/readiness/integration/teardown/budget drift and missing local opt-in (`Regression: #1494`)."
    ));
}

#[test]
fn regression_requires_local_fork_profile_preflight_guard_marker() {
    // Regression: #1648
    assert!(PLAN.contains(
        "local fork profile preflight lane fails closed for local opt-in, checkout/profile contract drift, probe command failures, and runtime budget overruns (`Regression: #1648`)."
    ));
}

#[test]
fn regression_requires_local_fork_profile_preflight_contract_lane_guard_marker() {
    // Regression: #1697
    assert!(PLAN.contains(
        "local fork profile preflight policy and contract-lane command/report drift remains fail-closed (`Regression: #1697`)."
    ));
}

#[test]
fn regression_requires_local_fork_self_test_guard_marker() {
    // Regression: #1652
    assert!(PLAN.contains(
        "local fork self-test lane fails closed for local opt-in, nested matrix/policy checkpoint failures, and runtime budget overruns (`Regression: #1652`)."
    ));
}

#[test]
fn regression_requires_local_fork_self_test_contract_lane_guard_marker() {
    // Regression: #1702
    assert!(PLAN.contains(
        "local fork self-test policy and contract-lane command/report drift remains fail-closed (`Regression: #1702`)."
    ));
}

#[test]
fn regression_requires_local_fork_portability_preflight_contract_lane_guard_marker() {
    // Regression: #1707
    assert!(PLAN.contains(
        "local fork portability preflight lane fails closed for local opt-in, mold linker drift, libudev dependency drift, and compile probe failures (`Regression: #1707`)."
    ));
}

#[test]
fn regression_requires_real_fork_local_process_wrapper_guard_marker() {
    // Regression: #1644
    assert!(PLAN.contains(
        "real-fork local process wrapper lane fails closed for local opt-in, serve-command profile drift, self-test/lifecycle/policy checkpoint failure, and runtime budget overruns (`Regression: #1644`)."
    ));
}

#[test]
fn regression_requires_real_fork_wrapper_bootstrap_prerequisite_guard_marker() {
    // Regression: #1667
    assert!(PLAN.contains(
        "real-fork local process wrapper bootstrap-first prerequisite ordering remains fail-closed for bootstrap lane/policy checkpoint drift (`Regression: #1667`)."
    ));
}

#[test]
fn regression_requires_real_fork_wrapper_policy_checker_guard_marker() {
    // Regression: #1671
    assert!(PLAN.contains(
        "real-fork local process wrapper policy checker lane remains fail-closed for schema/contracts/checkpoint drift (`Regression: #1671`)."
    ));
}

#[test]
fn plan_contains_runtime_transport_retry_reconnect_failure_taxonomy() {
    assert!(PLAN.contains("## Runtime Transport Retry-Reconnect Failure Taxonomy (Issue #4508)"));
    assert!(PLAN.contains("validate_live_transport_fault_matrix_live.sh"));
    assert!(
        PLAN.contains("live_transport_fault_matrix_policy_reason_codes_classification_mismatch")
    );
    assert!(PLAN.contains(
        "live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch"
    ));
    assert!(PLAN.contains(
        "live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch"
    ));
    assert!(PLAN.contains("cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout"));
    assert!(PLAN.contains(
        "peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1"
    ));
    assert!(PLAN.contains("peer_adapter_multi_process_validation_local_heavy_status=required"));
    assert!(PLAN.contains(
        "peer sender-integrity drift fixtures must fail closed with `p2p_transport_unknown_sender_peer` (`Regression: #4319`)."
    ));
    assert!(PLAN.contains(
        "retry-timeout classification must remain stable with `p2p_live_reconnect_retry_dial_timeout` before budget exhaustion and `p2p_live_reconnect_retry_budget_exhausted` only when retry budget is reached (`Regression: #4319`)."
    ));
}
