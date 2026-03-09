use super::super::*;
use super::support::{
    accept_task, build_task_escrow_snapshot, create_task, fund_escrow, release_escrow,
    unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-escrow-state");
    let state_file_text = state_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_text.as_str()));
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
    accept_task(&snapshot, bind_addr.as_str(), task_caller_did, 22, created_task.task_id.as_str());
    let queried_task = super::support::query_task(&snapshot, bind_addr.as_str(), task_caller_did, 23, created_task.task_id.as_str());
    let funded_escrow = fund_escrow(
        &snapshot,
        bind_addr.as_str(),
        escrow_caller_did,
        24,
        r#"{"task_id":"persisted-task","amount":1}"#,
    );
    let escrow_id = funded_escrow["escrow_id"].as_str().expect("escrow id should be string");
    let released_escrow = release_escrow(&snapshot, bind_addr.as_str(), escrow_caller_did, 25, escrow_id);

    assert_eq!(created_task.state, "submitted");
    assert_eq!(queried_task["task_id"], created_task.task_id);
    assert_eq!(queried_task["state"], "accepted");
    assert_eq!(funded_escrow["state"], "funded");
    assert_eq!(released_escrow["escrow_id"], escrow_id);
    assert_eq!(released_escrow["state"], "released");
    let _ = fs::remove_file(state_file);
}
