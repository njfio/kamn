use super::super::super::*;
use super::super::support::{
    build_task_escrow_snapshot, create_task, query_task, raw_create_task_response,
    raw_signed_request, register_agent_profile, set_audit_export_file_env, set_state_file_env,
    unique_named_state_file, SignedRequest,
};

pub(super) fn setup_dispatch_route_case() -> DispatchRouteCase {
    let env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-dispatch-state");
    let (_state_file_text, state_file_guard) = set_state_file_env(state_file.as_path());
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34116");
    let bind_addr = reserve_loopback_addr();
    let worker = register_agent_profile(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:task-worker-dispatch",
        301,
        r#"{"agent_type":"worker","model_family":"dispatch","capabilities":["image-analysis"]}"#,
    );
    DispatchRouteCase {
        _env: env,
        snapshot,
        state_file,
        _state_file_guard: state_file_guard,
        bind_addr,
        creator_did: "kamn:did:agent:task-creator-dispatch",
        worker_did: worker.did,
    }
}

pub(super) fn dispatch_task_to_registered_worker(
    dispatch: &DispatchRouteCase,
) -> (crate::service_api_endpoint::ServiceApiTaskCreateBody, Value) {
    let body = canonical_dispatch_body(
        dispatch.worker_did.as_str(),
        "image-analysis",
        "dispatch-create",
    );
    let created_task = create_task(
        &dispatch.snapshot,
        dispatch.bind_addr.as_str(),
        dispatch.creator_did,
        302,
        body.as_str(),
    );
    let queried_task = query_task(
        &dispatch.snapshot,
        dispatch.bind_addr.as_str(),
        dispatch.creator_did,
        303,
        created_task.task_id.as_str(),
    );
    (created_task, queried_task)
}

pub(super) fn assert_dispatched_task_state(
    created_task: &crate::service_api_endpoint::ServiceApiTaskCreateBody,
    queried_task: &Value,
    persisted_task: &Value,
    worker_did: &str,
) {
    assert_eq!(created_task.state, "submitted");
    assert_eq!(queried_task["state"], "submitted");
    assert_eq!(persisted_task["state"], "submitted");
    assert_eq!(persisted_task["assignee"], worker_did);
}

pub(super) fn setup_missing_worker_route_case() -> MissingWorkerRouteCase {
    let env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-dispatch-missing-worker");
    let (_state_file_text, state_file_guard) = set_state_file_env(state_file.as_path());
    MissingWorkerRouteCase {
        _env: env,
        snapshot: build_task_escrow_snapshot("127.0.0.1:34117"),
        state_file,
        _state_file_guard: state_file_guard,
        bind_addr: reserve_loopback_addr(),
        creator_did: "kamn:did:agent:task-creator-missing-worker",
    }
}

pub(super) fn query_missing_worker_task(missing_worker: &MissingWorkerRouteCase) -> String {
    let provider = register_agent_profile(
        &missing_worker.snapshot,
        missing_worker.bind_addr.as_str(),
        missing_worker.creator_did,
        400,
        r#"{"agent_type":"provider","model_family":"test","capabilities":["tasks"]}"#,
    );
    let body = canonical_dispatch_body(
        provider.did.as_str(),
        "vision-sync",
        "missing-worker-create",
    );
    let created_task = create_task(
        &missing_worker.snapshot,
        missing_worker.bind_addr.as_str(),
        missing_worker.creator_did,
        401,
        body.as_str(),
    );
    raw_signed_request(
        &missing_worker.snapshot,
        missing_worker.bind_addr.as_str(),
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/tasks/{}", created_task.task_id).as_str(),
            caller_did: missing_worker.creator_did,
            nonce: 402,
            body: "",
            extra_headers: &[],
        },
    )
}

fn canonical_dispatch_body(provider: &str, task_type: &str, key: &str) -> String {
    serde_json::json!({
        "provider_did": provider,
        "transaction_id": format!("transaction-{key}"),
        "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "idempotency_key": key,
        "task_type": task_type,
        "description": "canonical dispatch task",
    })
    .to_string()
}

pub(super) fn setup_audit_export_failure_route_case() -> AuditExportFailureRouteCase {
    let env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-audit-export-failure");
    let (_state_file_text, state_file_guard) = set_state_file_env(state_file.as_path());
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34117");
    let bind_addr = reserve_loopback_addr();
    let caller_did = "kamn:did:agent:test-client-task-audit-failure";
    let provider = register_agent_profile(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        50,
        r#"{"agent_type":"provider","model_family":"test","capabilities":["tasks"]}"#,
    );
    let invalid_export_file = std::env::temp_dir()
        .join(format!(
            "kamn-node-missing-audit-dir-{}",
            std::process::id()
        ))
        .join("audit-export.json");
    let (_audit_export_text, audit_export_guard) =
        set_audit_export_file_env(invalid_export_file.as_path());
    AuditExportFailureRouteCase {
        _env: env,
        snapshot,
        state_file,
        _state_file_guard: state_file_guard,
        _audit_export_guard: audit_export_guard,
        bind_addr,
        caller_did,
        provider_did: provider.did,
    }
}

pub(super) fn create_task_with_broken_audit_export(
    failure: &AuditExportFailureRouteCase,
) -> String {
    let body = canonical_dispatch_body(
        failure.provider_did.as_str(),
        "audit",
        "audit-export-failure",
    );
    raw_create_task_response(
        &failure.snapshot,
        failure.bind_addr.as_str(),
        failure.caller_did,
        51,
        body.as_str(),
    )
}

pub(super) struct DispatchRouteCase {
    _env: ServiceApiTestEnvGuards,
    pub(super) snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    pub(super) state_file: std::path::PathBuf,
    _state_file_guard: EnvVarGuard,
    pub(super) bind_addr: String,
    pub(super) creator_did: &'static str,
    pub(super) worker_did: String,
}

pub(super) struct MissingWorkerRouteCase {
    _env: ServiceApiTestEnvGuards,
    pub(super) snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    pub(super) state_file: std::path::PathBuf,
    _state_file_guard: EnvVarGuard,
    pub(super) bind_addr: String,
    pub(super) creator_did: &'static str,
}

pub(super) struct AuditExportFailureRouteCase {
    _env: ServiceApiTestEnvGuards,
    pub(super) snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    pub(super) state_file: std::path::PathBuf,
    _state_file_guard: EnvVarGuard,
    _audit_export_guard: EnvVarGuard,
    pub(super) bind_addr: String,
    caller_did: &'static str,
    provider_did: String,
}
