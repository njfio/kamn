const RUST_DOC: &str = include_str!("../../../docs/foundation/rust-sdk-alpha.md");
const PYTHON_DOC: &str = include_str!("../../../docs/foundation/python-sdk-beta.md");
const TYPESCRIPT_DOC: &str = include_str!("../../../docs/foundation/typescript-sdk-beta.md");

#[test]
fn docs_reference_shared_sdk_parity_fixture_source() {
    assert!(RUST_DOC.contains("fixtures/sdk_parity/register_validation_cases.json"));
    assert!(PYTHON_DOC.contains("fixtures/sdk_parity/register_validation_cases.json"));
    assert!(TYPESCRIPT_DOC.contains("fixtures/sdk_parity/register_validation_cases.json"));
}

#[test]
fn regression_requires_shared_matrix_command_in_all_sdk_docs() {
    // Regression: #583
    assert!(RUST_DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
    assert!(PYTHON_DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
    assert!(TYPESCRIPT_DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
}

#[test]
fn regression_requires_sdk_fixture_snapshot_drift_checker_commands() {
    // Regression: #940
    assert!(RUST_DOC.contains("register_validation_snapshot.json"));
    assert!(PYTHON_DOC.contains("register_validation_snapshot.json"));
    assert!(TYPESCRIPT_DOC.contains("register_validation_snapshot.json"));
    assert!(RUST_DOC.contains("run_example_fixture_drift_contract_lane.sh"));
    assert!(PYTHON_DOC.contains("run_example_fixture_drift_contract_lane.sh"));
    assert!(TYPESCRIPT_DOC.contains("run_example_fixture_drift_contract_lane.sh"));
}

#[test]
fn rust_doc_references_sdk_schema_shared_contract_script() {
    assert!(RUST_DOC.contains("sdk_schema_compatibility_contract.py"));
    assert!(RUST_DOC.contains("live_transport_smoke_parity_policy_contract.py"));
    assert!(RUST_DOC.contains("live_transport_smoke_parity_lane_contract.py"));
    assert!(RUST_DOC.contains("live_transport_smoke_parity_contract_lane_contract.py"));
    // Regression: #1182
    assert!(RUST_DOC.contains("`Regression: #1182`"));
    // Regression: #1186
    assert!(RUST_DOC.contains("`Regression: #1186`"));
    // Regression: #1190
    assert!(RUST_DOC.contains("`Regression: #1190`"));
    assert!(
        RUST_DOC.contains("smoke parity policy checker wrapper remains pinned to the shared contract implementation marker")
    );
    assert!(RUST_DOC.contains(
        "smoke parity lane wrapper remains pinned to the shared contract implementation marker"
    ));
    assert!(RUST_DOC.contains(
        "smoke parity contract lane wrapper remains pinned to the shared contract implementation marker"
    ));
}
