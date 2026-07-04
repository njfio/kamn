use crate::support::{bind_loopback_listener, run_bound_contract_server, ExpectedRequest};
use std::thread;

pub(crate) fn spawn_expected_server(
    expected_requests: Vec<ExpectedRequest>,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    let listener = bind_loopback_listener();
    let bind_addr = listener
        .local_addr()
        .expect("listener address should resolve")
        .to_string();
    let server = thread::spawn(move || run_bound_contract_server(listener, expected_requests));
    (bind_addr, server)
}
