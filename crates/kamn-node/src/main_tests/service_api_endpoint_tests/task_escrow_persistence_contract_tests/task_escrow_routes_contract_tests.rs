#[path = "task_escrow_routes_contract_tests/task_dispatch_support.rs"]
mod task_dispatch_support;

use super::super::*;
use super::support::{
    accept_task, build_task_escrow_snapshot, create_task, default_audit_export_file, fund_escrow,
    raw_create_task_response, read_audit_export_json, release_escrow, set_audit_export_file_env,
    set_state_file_env, unique_named_state_file,
};
use task_dispatch_support::{
    assert_dispatched_task_state, create_task_with_broken_audit_export,
    dispatch_task_to_registered_worker, query_missing_worker_task,
    setup_audit_export_failure_route_case, setup_dispatch_route_case,
    setup_missing_worker_route_case,
};

#[test]
fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-escrow-state");
    let (_state_file_text, _state_file_guard) = set_state_file_env(state_file.as_path());
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34106");
    let task_caller_did = "kamn:did:agent:test-client-task-state";
    let escrow_caller_did = "kamn:did:agent:test-client-escrow-state";
    let bind_addr = reserve_loopback_addr();

    let created_task = create_task(
        &snapshot,
        bind_addr.as_str(),
        task_caller_did,
        21,
        r#"{"title":"persisted-task","description":"task persistence contract"}"#,
    );
    accept_task(
        &snapshot,
        bind_addr.as_str(),
        task_caller_did,
        22,
        created_task.task_id.as_str(),
    );
    let queried_task = super::support::query_task(
        &snapshot,
        bind_addr.as_str(),
        task_caller_did,
        23,
        created_task.task_id.as_str(),
    );
    let funded_escrow = fund_escrow(
        &snapshot,
        bind_addr.as_str(),
        escrow_caller_did,
        24,
        r#"{"task_id":"persisted-task","amount":1}"#,
    );
    let escrow_id = funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string");
    let released_escrow = release_escrow(
        &snapshot,
        bind_addr.as_str(),
        escrow_caller_did,
        25,
        escrow_id,
    );

    assert_eq!(created_task.state, "submitted");
    assert_eq!(queried_task["task_id"], created_task.task_id);
    assert_eq!(queried_task["state"], "accepted");
    assert_eq!(funded_escrow["state"], "funded");
    assert_eq!(released_escrow["escrow_id"], escrow_id);
    assert_eq!(released_escrow["state"], "released");
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_task_create_populates_audit_export_bundle() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-audit-export");
    let (_state_file_text, _state_file_guard) = set_state_file_env(state_file.as_path());
    let audit_export_file = default_audit_export_file(state_file.as_path());
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34116");
    let bind_addr = reserve_loopback_addr();
    let caller_did = "kamn:did:agent:test-client-task-audit";

    let created_task = create_task(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        41,
        r#"{"title":"audit-task","description":"task audit export contract"}"#,
    );
    let audit_export = read_audit_export_json(audit_export_file.as_path());

    assert_task_create_audit_export(&created_task, &audit_export);
    let _ = fs::remove_file(audit_export_file);
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_task_create_fails_loud_when_audit_export_write_fails() {
    let failure = setup_audit_export_failure_route_case();
    let response = create_task_with_broken_audit_export(&failure);

    assert!(response.contains("HTTP/1.1 500 Internal Server Error"));
    assert!(response.contains("audit export"));
    let _ = fs::remove_file(failure.state_file);
}

#[test]
fn integration_service_api_endpoint_auto_dispatches_sdk_shaped_task_to_registered_worker() {
    let dispatch = setup_dispatch_route_case();
    let (created_task, queried_task) = dispatch_task_to_registered_worker(&dispatch);
    let persisted = super::support::read_state_json(dispatch.state_file.as_path());
    let task = &persisted["tasks"][created_task.task_id.as_str()];

    assert_dispatched_task_state(
        &created_task,
        &queried_task,
        task,
        dispatch.worker_did.as_str(),
    );
    let _ = fs::remove_file(dispatch.state_file);
}

#[test]
fn integration_service_api_endpoint_task_query_fails_loud_when_no_dispatch_worker_exists() {
    let missing_worker = setup_missing_worker_route_case();
    let response = query_missing_worker_task(&missing_worker);

    assert!(response.contains("HTTP/1.1 503 Service Unavailable"));
    assert!(response.contains("service_api_task_dispatch_prerequisites_missing"));
    assert!(response.contains("task dispatch prerequisites missing"));
    let _ = fs::remove_file(missing_worker.state_file);
}

#[test]
fn integration_service_api_endpoint_repeated_task_query_keeps_dispatched_task_terminal() {
    let dispatch = setup_dispatch_route_case();
    let (created_task, queried_task) = dispatch_task_to_registered_worker(&dispatch);
    let repeated_query = super::support::query_task(
        &dispatch.snapshot,
        dispatch.bind_addr.as_str(),
        dispatch.creator_did,
        304,
        created_task.task_id.as_str(),
    );
    let persisted = super::support::read_state_json(dispatch.state_file.as_path());
    let task = &persisted["tasks"][created_task.task_id.as_str()];

    assert_dispatched_task_state(
        &created_task,
        &queried_task,
        task,
        dispatch.worker_did.as_str(),
    );
    assert_eq!(repeated_query["state"], "completed");
    let _ = fs::remove_file(dispatch.state_file);
}

fn assert_task_create_audit_export(created_task: &ServiceApiTaskCreateBody, audit_export: &Value) {
    assert_eq!(created_task.state, "submitted");
    assert_eq!(audit_export["manifest"]["record_count"], 1);
    assert_eq!(audit_export["records"][0]["domain"], "Tasks");
    assert_eq!(
        audit_export["records"][0]["actor"],
        "kamn:did:agent:service-api-runtime"
    );
    assert_eq!(audit_export["records"][0]["event_id"], created_task.task_id);
    assert_eq!(
        audit_export["records"][0]["action"],
        "service_api_task_created"
    );
}
