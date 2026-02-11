const DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn doc_contains_make_and_demo_scope_contract_rules() {
    assert!(DOC.contains("make check"));
    assert!(DOC.contains("make test"));
    assert!(DOC.contains("make demo"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane_tests"));
    assert!(DOC.contains("sdk-live-localhost-integration"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane.sh"));
    assert!(DOC.contains("scripts/ci/select_targets.sh"));
    assert!(DOC.contains("run_kolme_version_compatibility_contract_tests=true"));
    assert!(DOC.contains("test_run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(DOC.contains("run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(DOC.contains("check_fast_gate_native_api_parity_policy.py"));
    assert!(DOC.contains("KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120"));
    assert!(DOC.contains("run_local_fork_sync_metadata_lane.sh --mode run"));
    assert!(DOC.contains("run_local_fork_smoke_evidence_lane.sh --mode run"));
    assert!(DOC.contains(
        "run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2"
    ));
    assert!(DOC.contains("run_local_kolme_api_smoke_lane.sh --mode run"));
    assert!(DOC.contains("run_local_kolme_live_api_conformance_harness.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json"
    ));
    assert!(DOC.contains("run_local_kamn_live_runtime_integration_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_process_lifecycle_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kolme_fork_process_lifecycle_contract_lane.sh --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json"
    ));
    assert!(DOC.contains("run_local_runtime_commit_live_lane.sh --mode run"));
    assert!(DOC.contains("run_local_native_api_parity_live_proof_lane.sh --mode run"));
    assert!(
        DOC.contains("--request PUT --data '{\\\"message\\\":\\\"native-parity\\\",\\\"signature\\\":\\\"sig\\\",\\\"recovery_id\\\":1}' http://127.0.0.1:3000/broadcast")
    );
    assert!(DOC.contains("test_run_local_runtime_commit_live_lane.sh"));
    assert!(DOC.contains("test_run_local_native_api_parity_live_proof_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_live_api_conformance_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_process_lifecycle_contract_lane.sh"));
    assert!(DOC.contains("run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(DOC.contains("test_run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60"));
    assert!(DOC.contains("run_local_bootstrap_health_checks.sh"));
    assert!(DOC.contains("run_local_e2e_integration_lane.sh"));
    assert!(DOC.contains("check_local_e2e_integration_policy.py"));
    assert!(DOC.contains("run_local_e2e_integration_contract_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_LOCAL_HEAVY=1"));
    assert!(
        DOC.contains("local-only heavy Kolme run-mode commands remain excluded from ci-fast-gate.")
    );
    assert!(DOC.contains(
        "local-only fork sync/smoke run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local Kolme API probe/smoke run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local live API conformance harness run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork bootstrap/readiness run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local KAMN live runtime integration run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork process lifecycle integration run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local runtime-commit live run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local native API parity live-proof run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "native parity fast/local command matrix remains synchronized across `README.md` and `docs/planning/kolme-devnet-ops.md`."
    ));
}

#[test]
fn regression_requires_make_and_selector_demo_contract_marker() {
    // Regression: #900
    assert!(DOC.contains("Regression: #900"));
    assert!(DOC.contains("make-target and selector workflow drift"));
    assert!(DOC.contains("Regression: #1419"));
    assert!(DOC.contains("Regression: #1431"));
    assert!(DOC.contains("Regression: #1682"));
    assert!(DOC.contains("Regression: #1441"));
    assert!(DOC.contains("Regression: #1451"));
    assert!(DOC.contains("Regression: #1467"));
    assert!(DOC.contains("Regression: #1468"));
    assert!(DOC.contains("Regression: #1482"));
    assert!(DOC.contains("Regression: #1483"));
    assert!(DOC.contains("Regression: #1488"));
    assert!(DOC.contains("Regression: #1489"));
    assert!(DOC.contains("Regression: #1494"));
    assert!(DOC.contains("Regression: #1462"));
    assert!(DOC.contains("Regression: #1466"));
    assert!(DOC.contains("Regression: #1497"));
}
