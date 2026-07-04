use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/working-vertical-slice.md";
const SERVICE_API_TEST_ROOT: &str = "crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs";
const VERTICAL_SLICE_TEST_MODULE_PATH: &str =
    "crates/kamn-node/src/main_tests/service_api_endpoint_tests/vertical_slice_contract_tests.rs";
const DOC_MARKERS: &[&str] = &[
    "# Working Vertical Slice",
    "two identities",
    "encrypted delivery",
    "task lifecycle transition",
    "audit export",
    "data_layer_runtime_evidence",
    "What This Proves",
    "What This Does Not Prove",
    "cargo test -p kamn-node",
];
const ROOT_MARKERS: &[&str] =
    &["#[path = \"service_api_endpoint_tests/vertical_slice_contract_tests.rs\"]"];
const TEST_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence",
    "X25519-XChaCha20-Poly1305",
    "service_api_task_created",
    "completed",
    "delivered",
];

#[test]
fn regression_working_vertical_slice_doc_exists_with_operator_markers() {
    let doc = read_workspace_file(DOC_PATH);
    for marker in DOC_MARKERS {
        assert!(
            doc.contains(marker),
            "working vertical slice doc missing required marker: {marker}"
        );
    }
}

#[test]
fn regression_service_api_test_root_wires_vertical_slice_module() {
    let root = read_workspace_file(SERVICE_API_TEST_ROOT);
    for marker in ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "service_api_endpoint_tests.rs missing vertical-slice marker: {marker}"
        );
    }
}

#[test]
fn regression_vertical_slice_integration_test_exists_with_required_markers() {
    let test_module = read_workspace_file(VERTICAL_SLICE_TEST_MODULE_PATH);
    for marker in TEST_MARKERS {
        assert!(
            test_module.contains(marker),
            "vertical slice integration module missing required marker: {marker}"
        );
    }
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    let path_display = path.display();
    assert!(path.exists(), "expected path to exist: {path_display}");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path_display}: {error}"))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}
