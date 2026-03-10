use super::*;

pub(crate) fn start_contract_server(
    request_budget: u64,
    expected_agent_sender_did: &str,
    expected_message_body: Option<String>,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let expected_sender = expected_agent_sender_did.to_owned();
    let server = thread::spawn(move || {
        run_live_transport_contract_server(
            server_addr,
            request_budget,
            expected_sender.as_str(),
            expected_message_body,
        )
    });
    wait_for_server_ready(bind_addr.as_str());
    (bind_addr, server)
}

pub(crate) fn assert_server_result(server: thread::JoinHandle<Result<(), String>>, message: &str) {
    let server_result = server.join().expect("server thread should join");
    assert!(server_result.is_ok(), "{message}");
}
