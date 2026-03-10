use super::support::assert_runtime_local_lane_contract_markers;

#[test]
fn doc_contains_runtime_local_signal_secret_hygiene_contract_lane_ci_mode_markers() {
    assert_runtime_local_lane_contract_markers(
        "## Runtime Local Signal/Secret Hygiene Contract Lane",
        &[
            "validate_local_signal_secret_hygiene_live.sh --mode dry-run --output-json /tmp/runtime-local-signal-secret-hygiene-summary.json",
            "KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_OPT_IN=1 bash scripts/runtime/validate_local_signal_secret_hygiene_live.sh --mode run --output-json /tmp/runtime-local-signal-secret-hygiene-summary.json",
            "check_local_signal_secret_hygiene_live_policy.sh --report-file /tmp/runtime-local-signal-secret-hygiene-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/runtime-local-signal-secret-hygiene-policy.json",
            "validate_local_signal_secret_hygiene_live_contract_lane.sh --output-json /tmp/runtime-local-signal-secret-hygiene-contract-lane-report.json --policy-output-json /tmp/runtime-local-signal-secret-hygiene-policy.json",
            "test_validate_local_signal_secret_hygiene_live.sh",
            "test_check_local_signal_secret_hygiene_live_policy.sh",
            "test_validate_local_signal_secret_hygiene_live_contract_lane.sh",
        ],
        "ci-local contract-lane boundary rejects `--max-seconds > 240`.",
        &[
            "shutdown_reason_taxonomy_version=kamn.runtime.local-signal-shutdown-reason-taxonomy.v1",
            "shutdown_reason_codes_csv=local_signal_shutdown_path_drift_detected,local_graceful_drain_bypass_detected,ci_local_signal_shutdown_budget_boundary_exceeded",
            "signal_graceful_drain_status=verified",
            "local signal/secret hygiene run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
            "fallback_signer_secret_present_violation",
            "local_signal_shutdown_path_drift_detected",
            "local_graceful_drain_bypass_detected",
            "ci_local_signal_shutdown_budget_boundary_exceeded",
        ],
        "runtime local signal hygiene",
    );
}

#[test]
fn doc_contains_runtime_local_metrics_scrape_contract_lane_ci_mode_markers() {
    assert_runtime_local_lane_contract_markers(
        "## Runtime Local Metrics Scrape Contract Lane",
        &[
            "validate_local_metrics_scrape_live.sh --mode dry-run --output-json /tmp/local-metrics-scrape-live-summary.json",
            "KAMN_LOCAL_METRICS_SCRAPE_OPT_IN=1 bash scripts/runtime/validate_local_metrics_scrape_live.sh --mode run --output-json /tmp/local-metrics-scrape-live-summary.json",
            "check_local_metrics_scrape_live_policy.sh --report-file /tmp/local-metrics-scrape-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-metrics-scrape-live-policy.json",
            "validate_local_metrics_scrape_live_contract_lane.sh --output-json /tmp/local-metrics-scrape-live-contract-lane-report.json --policy-output-json /tmp/local-metrics-scrape-live-policy.json",
            "test_validate_local_metrics_scrape_live_contract_lane.sh",
            "test_check_local_metrics_scrape_live_policy.sh",
        ],
        "local metrics scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &[
            "local_metrics_scrape_policy_marker_missing:local_scrape_probe_status",
            "local_metrics_scrape_policy_marker_missing:scrape_latency_budget_status",
            "local_metrics_scrape_policy_metrics_emission_reason_taxonomy_version_mismatch",
        ],
        "runtime local metrics scrape",
    );
}
