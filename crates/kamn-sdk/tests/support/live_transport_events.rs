#[path = "live_transport_http.rs"]
mod http;

use std::net::TcpListener;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

pub(crate) use self::http::parse_http_request;

const CHAIN_ID: &str = "kamn-sdk-live";
const CHAIN_VERSION: &str = "1";
const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn ensure_live_test_env() {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    std::env::set_var(LIVE_CHAIN_ID_ENV, CHAIN_ID);
    std::env::set_var(LIVE_CHAIN_VERSION_ENV, CHAIN_VERSION);
    std::env::set_var(LIVE_REQUESTER_DID_ENV, "kamn:did:agent:live-events");
}

pub(crate) fn with_env_lock<T>(callback: impl FnOnce() -> T) -> T {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = LOCK.get_or_init(|| Mutex::new(()));
    let guard = lock.lock().expect("env lock should not be poisoned");
    let output = callback();
    drop(guard);
    output
}

pub(crate) fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(40));
}
