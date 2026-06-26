use crate::support::{assert_not_contains_all, read_repo_file};

#[test]
fn main_module_extraction_contract_removes_inline_kolme_live_branch_execution_impls() {
    let main_rs = read_repo_file("src/main.rs");
    let runtime_orchestration_rs = read_repo_file("src/runtime_orchestration.rs");
    let runtime_mode_handlers_rs =
        read_repo_file("src/runtime_orchestration/runtime_mode_handlers.rs");

    assert_not_contains_all(
        &main_rs,
        &[
            (
                "KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(",
                "main.rs should not keep inline Kolme live provider constructor path",
            ),
            (
                "submit_runtime_commit(signed_wire_payload.as_str(), request.idempotency_key())",
                "main.rs should not keep inline Kolme live submit invocation",
            ),
            (
                "KolmeRuntimeCommitFinalityChecker::new(",
                "main.rs should not keep inline Kolme live finality checker orchestration",
            ),
        ],
    );
    assert!(
        runtime_orchestration_rs.contains("execute_kolme_live_runtime(")
            || runtime_mode_handlers_rs.contains("execute_kolme_live_runtime("),
        "runtime orchestration boundary should delegate Kolme live runtime execution"
    );
}

#[test]
fn main_module_extraction_contract_delegates_endpoint_orchestration_to_runtime_entrypoint_module() {
    let main_rs = read_repo_file("src/main.rs");
    let runtime_entrypoint_rs = read_repo_file("src/runtime_entrypoint.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            (
                "fn classify_service_api_endpoint_runtime_path(",
                "main.rs should not keep inline service-api endpoint runtime-path classifier",
            ),
            (
                "fn should_skip_observability_endpoint_for_full_supervisor(",
                "main.rs should not keep inline observability endpoint runtime-path guard",
            ),
        ],
    );
    assert!(
        main_rs.contains("serve_runtime_endpoints("),
        "main.rs should delegate endpoint orchestration to runtime_entrypoint module"
    );
    assert!(
        runtime_entrypoint_rs.contains("pub(crate) fn serve_runtime_endpoints("),
        "runtime_entrypoint module should own endpoint-serving orchestration"
    );
    assert!(
        runtime_entrypoint_rs.contains("ServiceApiEndpointRuntimePath"),
        "runtime_entrypoint module should own endpoint runtime-path classification"
    );
}
