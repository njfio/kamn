#[path = "route_family_contract_tests/registration_task_bridge_contract_tests.rs"]
mod registration_task_bridge_contract_tests;
#[path = "route_family_contract_tests/replay_and_channel_contract_tests.rs"]
mod replay_and_channel_contract_tests;

#[test]
fn regression_service_api_client_rejects_replayed_nonce() {
    replay_and_channel_contract_tests::assert_replay_nonce_contract();
}

#[test]
fn spec_c01_service_api_client_lists_channel_messages_through_route_contract() {
    replay_and_channel_contract_tests::assert_channel_messages_contract();
}

#[test]
fn regression_service_api_client_registration_surface_contract_exists() {
    registration_task_bridge_contract_tests::assert_registration_surface_contract();
}

#[test]
fn spec_c02_service_api_client_executes_task_transition_and_escrow_route_contracts() {
    registration_task_bridge_contract_tests::assert_task_and_escrow_routes();
}

#[test]
fn spec_c03_service_api_client_executes_bridge_route_contracts() {
    registration_task_bridge_contract_tests::assert_bridge_routes();
}
