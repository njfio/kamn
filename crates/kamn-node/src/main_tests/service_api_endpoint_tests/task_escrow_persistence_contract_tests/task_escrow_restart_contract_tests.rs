use super::super::*;
use super::support::{
    accept_task, build_task_escrow_snapshot, complete_task, create_task, fund_escrow, query_task,
    read_state_json, release_escrow, set_state_file_env, unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_persists_task_and_escrow_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-task-escrow-restart-state");
    let (_state_file_text, _state_file_guard) = set_state_file_env(state_file.as_path());
    let task_caller_did = "kamn:did:agent:test-client-task-restart";
    let escrow_caller_did = task_caller_did;
    let first_snapshot = build_task_escrow_snapshot("127.0.0.1:34110");
    let bind_addr = reserve_loopback_addr();

    let created_task = create_task(
        &first_snapshot,
        bind_addr.as_str(),
        task_caller_did,
        61,
        r#"{"title":"restart-task","description":"persist restart"}"#,
    );
    accept_task(
        &first_snapshot,
        bind_addr.as_str(),
        task_caller_did,
        62,
        created_task.task_id.as_str(),
    );
    let funded_escrow = fund_escrow(
        &first_snapshot,
        bind_addr.as_str(),
        escrow_caller_did,
        63,
        format!(r#"{{"task_id":"{}","amount":5}}"#, created_task.task_id).as_str(),
    );
    let escrow_id = funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string");
    complete_task(
        &first_snapshot,
        bind_addr.as_str(),
        task_caller_did,
        64,
        created_task.task_id.as_str(),
    );
    release_escrow(
        &first_snapshot,
        bind_addr.as_str(),
        escrow_caller_did,
        65,
        escrow_id,
    );

    let restart_snapshot = build_task_escrow_snapshot("127.0.0.1:34111");
    let queried_task = query_task(
        &restart_snapshot,
        reserve_loopback_addr().as_str(),
        task_caller_did,
        66,
        created_task.task_id.as_str(),
    );
    let state_json = read_state_json(state_file.as_path());

    assert_eq!(queried_task["task_id"], created_task.task_id);
    assert_eq!(queried_task["state"], "completed");
    assert_eq!(
        state_json["tasks"][created_task.task_id.as_str()]["state"],
        "completed"
    );
    assert_eq!(
        state_json["escrows"][escrow_id]["state"],
        "release-authorized"
    );
    assert_eq!(
        state_json["escrow_transition_receipts"]
            .as_array()
            .expect("escrow receipts should survive restart")
            .len(),
        2
    );
    assert!(state_json["escrows"][escrow_id]["settlement_tx_signature"].is_null());
    let _ = fs::remove_file(state_file);
}
