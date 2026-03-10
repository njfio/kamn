use std::fs;
use std::path::{Path, PathBuf};

const SUPPORT_FILE: &str = "src/main_tests/observability_endpoint_tests/support.rs";
const TLS_SUPPORT_FILE: &str = "src/main_tests/observability_endpoint_tests/support/tls_support.rs";
const TRANSPORT_SUPPORT_FILE: &str = "src/main_tests/observability_endpoint_tests/support/transport_support.rs";
const ASYNC_REGRESSION_FILE: &str = "src/main_tests/observability_endpoint_tests/async_regression_contract_tests.rs";
const NEGATIVE_PATH_FILE: &str = "src/main_tests/observability_endpoint_tests/async_regression_contract_tests/negative_path_contract_tests.rs";
const STREAM_SERVER_FILE: &str = "src/main_tests/observability_endpoint_tests/stream_runtime_contract_tests/stream_server_contract_tests.rs";
const TLS_SUPPORT_MARKERS: &[&str] = &[
    "use std::io::{ErrorKind, Read, Write};",
    "use std::sync::Arc;",
    "use std::thread;",
];
const TRANSPORT_SUPPORT_MARKERS: &[&str] = &[
    "use std::io::{ErrorKind, Read, Write};",
    "use std::thread;",
];
const STREAM_SERVER_MARKERS: &[&str] = &["use std::sync::Arc;", "use std::thread;"];

fn repo_file(path: &str) -> String {
    let full_path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

fn assert_file_contains_markers(path: &str, markers: &[&str]) {
    let source = repo_file(path);
    for marker in markers {
        assert!(
            source.contains(marker),
            "{path} should declare explicit import marker: {marker}"
        );
    }
}

#[test]
fn regression_observability_endpoint_support_modules_restore_explicit_std_imports() {
    assert_file_contains_markers(SUPPORT_FILE, &["use std::net::TcpListener;", "use std::thread;"]);
    assert_file_contains_markers(TLS_SUPPORT_FILE, TLS_SUPPORT_MARKERS);
    assert_file_contains_markers(TRANSPORT_SUPPORT_FILE, TRANSPORT_SUPPORT_MARKERS);
}

#[test]
fn regression_observability_endpoint_leaf_modules_restore_explicit_thread_and_arc_imports() {
    let async_regression = repo_file(ASYNC_REGRESSION_FILE);
    assert!(
        async_regression.contains("use std::thread;"),
        "async_regression_contract_tests.rs should declare std::thread explicitly"
    );

    let negative_path = repo_file(NEGATIVE_PATH_FILE);
    assert!(
        negative_path.contains("use std::thread;"),
        "negative_path_contract_tests.rs should declare std::thread explicitly"
    );

    assert_file_contains_markers(STREAM_SERVER_FILE, STREAM_SERVER_MARKERS);
}
