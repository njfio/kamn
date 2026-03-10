use crate::support::{run_contract_server, ExpectedRequest};
use std::thread;

pub(crate) fn spawn_expected_server(
    bind_addr: String,
    expected_requests: Vec<ExpectedRequest>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || run_contract_server(bind_addr, expected_requests))
}
