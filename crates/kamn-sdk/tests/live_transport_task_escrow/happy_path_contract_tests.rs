use crate::support::{
    assert_escrow_flow, assert_task_flow, ensure_live_test_env, live_client,
    reserve_loopback_addr, spawn_expected_server, task_and_escrow_requests,
    wait_for_server_ready,
};
use kamn_sdk::KamnAgent;

#[test]
fn spec_c06_live_transport_task_and_escrow_routes_execute_network_contract() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server = spawn_expected_server(bind_addr.clone(), task_and_escrow_requests());
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    assert_task_flow(&mut client);
    let escrow_id = assert_escrow_flow(&mut client);
    client.release_escrow(&escrow_id).expect("release_escrow should succeed");

    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "task/escrow route server should satisfy request budget");
}
