use std::fs;

const ROOT_FILE: &str = "src/main_tests/observability_endpoint_tests.rs";
const ROOT_MAX_LINES: usize = 200;
const FILE_MAX_LINES: usize = 200;

const MODULE_DECLARATIONS: &[&str] = &[
    "mod payload_contract_tests;",
    "mod runtime_projection_contract_tests;",
    "mod endpoint_runtime_contract_tests;",
    "mod tls_contract_tests;",
    "mod stream_runtime_contract_tests;",
    "mod async_regression_contract_tests;",
    "mod support;",
];

const ROOT_REMOVED_MARKERS: &[&str] = &[
    "fn reserve_loopback_addr() -> String",
    "struct TestSkipServerVerification",
    "fn send_http_get(addr: &str, path: &str) -> String",
    "fn send_https_get(addr: &str, path: &str) -> String",
    "fn try_send_https_get(addr: &str, path: &str) -> Result<String, String>",
    "fn try_send_http_get(addr: &str, path: &str) -> Result<String, String>",
    "fn parse_args_with_clean_daemon_env(args: Vec<String>) -> Result<crate::NodeCli, ConfigError>",
    "fn send_raw_http_request(addr: &str, request: &str) -> String",
    "fn wait_for_endpoint_ready(addr: &str)",
    "fn wait_for_https_endpoint_ready(addr: &str)",
    "fn set_tls_mode_override_for_current_thread(",
    "fn observability_tls_temp_path(label: &str) -> String",
    "fn sample_observability_snapshot() -> RuntimeObservabilitySnapshot",
    "fn spec_c01_observability_endpoint_contract_checker_accepts_valid_surface_payloads()",
    "fn unit_observability_endpoint_maps_daemon_telemetry_into_snapshot()",
    "fn integration_runtime_observability_endpoint_serves_metrics_and_health_paths()",
    "fn integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes()",
    "fn integration_runtime_observability_endpoint_serves_stream_path()",
    "fn integration_runtime_observability_endpoint_returns_not_found_for_unknown_path()",
    "fn regression_observability_endpoint_keeps_async_negative_matrix_contracts()",
];

const EXPECTED_FILES_AND_MARKERS: &[(&str, &[&str])] = &[
    (
        "src/main_tests/observability_endpoint_tests/payload_contract_tests.rs",
        &[
            "fn spec_c01_observability_endpoint_contract_checker_accepts_valid_surface_payloads()",
            "fn spec_c05_observability_endpoint_contract_checker_fails_closed_with_stable_reason_markers()",
        ],
    ),
    (
        "src/main_tests/observability_endpoint_tests/runtime_projection_contract_tests.rs",
        &[
            "fn unit_observability_endpoint_rejects_metrics_path_without_leading_slash()",
            "fn unit_observability_endpoint_maps_daemon_telemetry_into_snapshot()",
        ],
    ),
    (
        "src/main_tests/observability_endpoint_tests/endpoint_runtime_contract_tests.rs",
        &[
            "fn functional_observability_endpoint_renders_metrics_and_health_payloads()",
            "fn integration_runtime_observability_endpoint_serves_metrics_and_health_paths()",
        ],
    ),
    (
        "src/main_tests/observability_endpoint_tests/tls_contract_tests.rs",
        &[
            "fn integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes()",
            "fn integration_runtime_observability_endpoint_tls_mode_rejects_plain_http_handshake()",
        ],
    ),
    (
        "src/main_tests/observability_endpoint_tests/stream_runtime_contract_tests.rs",
        &[
            "fn functional_observability_endpoint_renders_stream_payload()",
            "fn integration_runtime_observability_endpoint_supports_stream_reconnect_churn_sequence()",
        ],
    ),
    (
        "src/main_tests/observability_endpoint_tests/async_regression_contract_tests.rs",
        &[
            "fn integration_runtime_observability_endpoint_returns_not_found_for_unknown_path()",
            "fn regression_observability_endpoint_keeps_async_negative_matrix_contracts()",
        ],
    ),
    (
        "src/main_tests/observability_endpoint_tests/support.rs",
        &[
            "fn reserve_loopback_addr() -> String",
            "fn sample_observability_snapshot() -> RuntimeObservabilitySnapshot",
        ],
    ),
];

fn read_repo_file(path: &str) -> String {
    let full_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
    fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_markers_present(source: &str, markers: &[&str], path: &str) {
    for marker in markers {
        assert!(source.contains(marker), "{path} should contain marker: {marker}");
    }
}

fn assert_markers_absent(source: &str, markers: &[&str], path: &str) {
    for marker in markers {
        assert!(!source.contains(marker), "{path} should not contain marker: {marker}");
    }
}

#[test]
fn spec_c01_observability_endpoint_root_declares_split_submodules() {
    let source = read_repo_file(ROOT_FILE);
    assert_markers_present(&source, MODULE_DECLARATIONS, ROOT_FILE);
}

#[test]
fn spec_c02_observability_endpoint_root_removes_moved_helpers_and_tests() {
    let source = read_repo_file(ROOT_FILE);
    assert_markers_absent(&source, ROOT_REMOVED_MARKERS, ROOT_FILE);
}

#[test]
fn spec_c03_observability_endpoint_split_files_exist_and_own_coverage() {
    for (path, markers) in EXPECTED_FILES_AND_MARKERS {
        let source = read_repo_file(path);
        assert_markers_present(&source, markers, path);
    }
}

#[test]
fn spec_c04_observability_endpoint_root_respects_final_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "observability_endpoint_tests.rs should be <= {ROOT_MAX_LINES} lines, found {line_count}"
    );
}

#[test]
fn spec_c05_observability_endpoint_split_files_respect_line_budget() {
    for (path, _) in EXPECTED_FILES_AND_MARKERS {
        let line_count = read_repo_file(path).lines().count();
        assert!(
            line_count <= FILE_MAX_LINES,
            "{path} should be <= {FILE_MAX_LINES} lines, found {line_count}"
        );
    }
}
