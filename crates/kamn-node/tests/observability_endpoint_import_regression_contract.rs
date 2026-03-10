use std::fs;
use std::path::{Path, PathBuf};

const SUPPORT_FILE: &str = "src/main_tests/observability_endpoint_tests/support.rs";
const TLS_SUPPORT_FILE: &str = "src/main_tests/observability_endpoint_tests/support/tls_support.rs";
const TRANSPORT_SUPPORT_FILE: &str = "src/main_tests/observability_endpoint_tests/support/transport_support.rs";
const ASYNC_REGRESSION_FILE: &str = "src/main_tests/observability_endpoint_tests/async_regression_contract_tests.rs";
const NEGATIVE_PATH_FILE: &str = "src/main_tests/observability_endpoint_tests/async_regression_contract_tests/negative_path_contract_tests.rs";
const STREAM_SERVER_FILE: &str = "src/main_tests/observability_endpoint_tests/stream_runtime_contract_tests/stream_server_contract_tests.rs";

fn repo_file(path: &str) -> String {
    let full_path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

#[test]
fn regression_observability_endpoint_support_modules_restore_explicit_std_imports() {
    let support = repo_file(SUPPORT_FILE);
    assert!(support.contains("use std::net::TcpListener;"));
    assert!(support.contains("use std::thread;"));

    let tls_support = repo_file(TLS_SUPPORT_FILE);
    for marker in [
        "use std::io::{ErrorKind, Read, Write};",
        "use std::sync::Arc;",
        "use std::thread;",
    ] {
        assert!(
            tls_support.contains(marker),
            "tls_support.rs should declare explicit import marker: {marker}"
        );
    }

    let transport_support = repo_file(TRANSPORT_SUPPORT_FILE);
    for marker in [
        "use std::io::{ErrorKind, Read, Write};",
        "use std::thread;",
    ] {
        assert!(
            transport_support.contains(marker),
            "transport_support.rs should declare explicit import marker: {marker}"
        );
    }
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

    let stream_server = repo_file(STREAM_SERVER_FILE);
    for marker in ["use std::sync::Arc;", "use std::thread;"] {
        assert!(
            stream_server.contains(marker),
            "stream_server_contract_tests.rs should declare explicit import marker: {marker}"
        );
    }
}
