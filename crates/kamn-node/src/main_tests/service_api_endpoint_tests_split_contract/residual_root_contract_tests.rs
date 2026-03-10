use super::support::*;

#[test]
fn spec_c49_service_api_endpoint_root_file_removes_residual_root_tests() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn regression_service_api_env_lock_recovers_from_signer_lock_poison() {",
        "fn unit_service_api_endpoint_serde_payload_roundtrip_contracts() {",
        "fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {",
        "fn regression_service_api_payload_parse_reason_codes_fail_closed() {",
        "fn regression_service_api_endpoint_rejects_unknown_task_and_escrow_resource_transitions() {",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved residual test marker: {marker}"
        );
    }
}

#[test]
fn spec_c50_service_api_endpoint_residual_root_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(RESIDUAL_ROOT_MODULE_FILE);
    let env_lock = read_repo_file(ENV_LOCK_FILE);
    let serde_payload = read_repo_file(SERDE_PAYLOAD_FILE);
    let error_envelope = read_repo_file(ERROR_ENVELOPE_FILE);
    let payload_parse = read_repo_file(PAYLOAD_PARSE_FILE);
    let missing_resource = read_repo_file(MISSING_RESOURCE_FILE);

    assert_residual_root_module_markers(module_source.as_str());
    assert_residual_root_leaf_markers(
        env_lock.as_str(),
        serde_payload.as_str(),
        error_envelope.as_str(),
        payload_parse.as_str(),
        missing_resource.as_str(),
    );
}

fn assert_residual_root_module_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "mod env_lock_contract_tests;",
            "mod serde_payload_contract_tests;",
            "mod error_envelope_contract_tests;",
            "mod payload_parse_contract_tests;",
            "mod missing_resource_contract_tests;",
        ],
        "residual-root module",
    );
}

fn assert_residual_root_leaf_markers(
    env_lock: &str,
    serde_payload: &str,
    error_envelope: &str,
    payload_parse: &str,
    missing_resource: &str,
) {
    assert_env_lock_residual_markers(env_lock);
    assert_serde_residual_markers(serde_payload);
    assert_error_envelope_residual_markers(error_envelope);
    assert_payload_parse_residual_markers(payload_parse);
    assert_missing_resource_residual_markers(missing_resource);
}

fn assert_env_lock_residual_markers(source: &str) {
    assert_file_markers(
        source,
        &["fn regression_service_api_env_lock_recovers_from_signer_lock_poison() {"],
        "env-lock residual test file",
    );
}

fn assert_serde_residual_markers(source: &str) {
    assert_file_markers(
        source,
        &["fn unit_service_api_endpoint_serde_payload_roundtrip_contracts() {"],
        "serde residual test file",
    );
}

fn assert_error_envelope_residual_markers(source: &str) {
    assert_file_markers(
        source,
        &["fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {"],
        "error-envelope residual test file",
    );
}

fn assert_payload_parse_residual_markers(source: &str) {
    assert_file_markers(
        source,
        &["fn regression_service_api_payload_parse_reason_codes_fail_closed() {"],
        "payload-parse residual test file",
    );
}

fn assert_missing_resource_residual_markers(source: &str) {
    assert_file_markers(
        source,
        &["fn regression_service_api_endpoint_rejects_unknown_task_and_escrow_resource_transitions() {"],
        "missing-resource residual test file",
    );
}

#[test]
fn spec_c51_service_api_endpoint_root_declares_residual_root_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod residual_root_contract_tests;"),
        "service_api_endpoint_tests.rs should declare residual-root submodule"
    );
}

#[test]
fn spec_c52_service_api_endpoint_residual_root_split_files_stay_below_budget() {
    for path in [
        RESIDUAL_ROOT_MODULE_FILE,
        ENV_LOCK_FILE,
        SERDE_PAYLOAD_FILE,
        ERROR_ENVELOPE_FILE,
        PAYLOAD_PARSE_FILE,
        MISSING_RESOURCE_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
