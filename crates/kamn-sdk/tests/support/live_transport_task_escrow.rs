#[path = "live_transport_contract_server.rs"]
mod contract_server;

use kamn_sdk::AgentDid;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

const CHAIN_ID: &str = "kamn-sdk-live";
const CHAIN_VERSION: &str = "1";
const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(crate) struct ExpectedRequest {
    pub(crate) method: &'static str,
    pub(crate) path: String,
    pub(crate) body: String,
    pub(crate) sender_did: String,
    pub(crate) scope: &'static str,
    pub(crate) response_status: u16,
    pub(crate) response_body: String,
}

impl Default for ExpectedRequest {
    fn default() -> Self {
        Self {
            method: "POST",
            path: String::new(),
            body: String::new(),
            sender_did: String::new(),
            scope: "",
            response_status: 200,
            response_body: String::new(),
        }
    }
}

pub(crate) fn did(identifier: &str) -> AgentDid {
    AgentDid::parse(format!("kamn:did:agent:{identifier}")).expect("did should parse")
}

pub(crate) fn ensure_live_test_env() {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    std::env::set_var(LIVE_CHAIN_ID_ENV, CHAIN_ID);
    std::env::set_var(LIVE_CHAIN_VERSION_ENV, CHAIN_VERSION);
    std::env::set_var(LIVE_REQUESTER_DID_ENV, "kamn:did:agent:live-requester");
}

#[allow(dead_code)]
pub(crate) fn reserve_loopback_addr() -> String {
    let listener = bind_loopback_listener();
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn bind_loopback_listener() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener nonblocking mode should configure");
    listener
}

#[allow(dead_code)]
pub(crate) fn run_contract_server(
    bind_addr: String,
    expected_requests: Vec<ExpectedRequest>,
) -> Result<(), String> {
    contract_server::run_contract_server(bind_addr, expected_requests)
}

#[allow(dead_code)]
pub(crate) fn run_bound_contract_server(
    listener: TcpListener,
    expected_requests: Vec<ExpectedRequest>,
) -> Result<(), String> {
    contract_server::run_bound_contract_server(listener, expected_requests)
}

#[allow(dead_code)]
pub(crate) fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(40));
}

pub(crate) fn expected_request(method: &'static str, path: &str, body: &str) -> ExpectedRequest {
    ExpectedRequest {
        method,
        path: path.to_owned(),
        body: body.to_owned(),
        ..Default::default()
    }
}
