use super::support::assert_runtime_lane_markers;

#[test]
fn doc_contains_runtime_service_api_graceful_shutdown_drain_contract_lane_ci_mode_markers() {
    assert_runtime_lane_markers(
        "## Runtime Service API Graceful-Shutdown Drain Contract Lane",
        &[
            "validate_service_api_graceful_shutdown_drain_live.sh --mode dry-run --output-json /tmp/service-api-graceful-shutdown-drain-live-summary.json",
            "KAMN_LOCAL_GRACEFUL_SHUTDOWN_DRAIN_OPT_IN=1 bash scripts/runtime/validate_service_api_graceful_shutdown_drain_live.sh --mode run --output-json /tmp/service-api-graceful-shutdown-drain-live-summary.json",
            "check_service_api_graceful_shutdown_drain_live_policy.sh --report-file /tmp/service-api-graceful-shutdown-drain-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-graceful-shutdown-drain-policy.json",
            "validate_service_api_graceful_shutdown_drain_live_contract_lane.sh --output-json /tmp/service-api-graceful-shutdown-drain-contract-lane-report.json --policy-output-json /tmp/service-api-graceful-shutdown-drain-policy.json",
            "test_validate_service_api_graceful_shutdown_drain_live_contract_lane.sh",
            "test_check_service_api_graceful_shutdown_drain_live_policy.sh",
        ],
        "service api graceful-shutdown drain contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_graceful_shutdown_drain_policy_marker_missing:websocket_drain_status"],
        "service api graceful shutdown drain",
    );
}

#[test]
fn doc_contains_runtime_service_api_shutdown_abrupt_close_regression_contract_lane_ci_mode_markers() {
    assert_runtime_lane_markers(
        "## Runtime Service API Shutdown Abrupt-Close Regression Contract Lane",
        &[
            "validate_service_api_shutdown_abrupt_close_regression_live.sh --mode dry-run --output-json /tmp/service-api-shutdown-abrupt-close-regression-live-summary.json",
            "KAMN_LOCAL_SHUTDOWN_ABRUPT_CLOSE_REGRESSION_OPT_IN=1 bash scripts/runtime/validate_service_api_shutdown_abrupt_close_regression_live.sh --mode run --output-json /tmp/service-api-shutdown-abrupt-close-regression-live-summary.json",
            "check_service_api_shutdown_abrupt_close_regression_live_policy.sh --report-file /tmp/service-api-shutdown-abrupt-close-regression-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-shutdown-abrupt-close-regression-policy.json",
            "validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh --output-json /tmp/service-api-shutdown-abrupt-close-regression-contract-lane-report.json --policy-output-json /tmp/service-api-shutdown-abrupt-close-regression-policy.json",
            "test_validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh",
            "test_check_service_api_shutdown_abrupt_close_regression_live_policy.sh",
        ],
        "service api shutdown abrupt-close regression contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_shutdown_abrupt_close_regression_policy_marker_missing:abrupt_close_guard_status"],
        "service api shutdown abrupt close regression",
    );
}

#[test]
fn doc_contains_runtime_service_api_prometheus_metrics_contract_lane_ci_mode_markers() {
    assert_runtime_lane_markers(
        "## Runtime Service API Prometheus Metrics Contract Lane",
        &[
            "validate_service_api_prometheus_metrics_live.sh --mode dry-run --output-json /tmp/service-api-prometheus-metrics-live-summary.json",
            "KAMN_LOCAL_PROMETHEUS_METRICS_OPT_IN=1 bash scripts/runtime/validate_service_api_prometheus_metrics_live.sh --mode run --output-json /tmp/service-api-prometheus-metrics-live-summary.json",
            "check_service_api_prometheus_metrics_live_policy.sh --report-file /tmp/service-api-prometheus-metrics-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-prometheus-metrics-policy.json",
            "validate_service_api_prometheus_metrics_live_contract_lane.sh --output-json /tmp/service-api-prometheus-metrics-contract-lane-report.json --policy-output-json /tmp/service-api-prometheus-metrics-policy.json",
            "test_validate_service_api_prometheus_metrics_live_contract_lane.sh",
            "test_check_service_api_prometheus_metrics_live_policy.sh",
        ],
        "service api prometheus metrics contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_prometheus_metrics_policy_marker_missing:metrics_contract_status"],
        "service api prometheus metrics",
    );
}
