use super::*;

pub(crate) fn start_contract_server(
    request_budget: u64,
    expected_agent_sender_did: &str,
    expected_message_body: Option<String>,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    ensure_live_test_env();
    let listener = bind_loopback_listener();
    let bind_addr = listener
        .local_addr()
        .expect("local addr should resolve")
        .to_string();
    let expected_sender = expected_agent_sender_did.to_owned();
    let server = thread::spawn(move || {
        run_bound_live_transport_contract_server(
            listener,
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
