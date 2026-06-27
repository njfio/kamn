use crate::support::{
    assert_balance_route_fails_closed, assert_unknown_escrow_alias, assert_unknown_task_aliases,
    create_task_request, ensure_live_test_env, live_artifact, live_client, live_task,
    spawn_expected_server,
};
use kamn_sdk::{KamnAgent, SdkError};

#[test]
fn regression_live_transport_unknown_task_and_escrow_aliases_fail_closed() {
    ensure_live_test_env();
    let mut client = live_client("127.0.0.1:65535");
    assert_unknown_task_aliases(&mut client);
    assert_unknown_escrow_alias(&mut client);
    assert_balance_route_fails_closed(&client);
}

#[test]
fn regression_live_transport_submit_artifact_requires_accepted_task() {
    ensure_live_test_env();
    let (bind_addr, server) = spawn_expected_server(vec![create_task_request()]);

    let mut client = live_client(bind_addr.as_str());
    let task_id = client
        .create_task(live_task())
        .expect("create_task should succeed");
    assert_eq!(
        client.submit_artifact(&task_id, live_artifact()),
        Err(SdkError::Conflict(
            "task must be accepted before artifact submission"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "unaccepted task artifact submission should not emit an extra network request"
    );
}
