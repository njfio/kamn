use super::*;

pub(crate) const CHAIN_ID: &str = "kamn-sdk-live";
pub(crate) const CHAIN_VERSION: &str = "1";
const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
pub(crate) const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
pub(crate) const DEFAULT_LIVE_REQUESTER_DID: &str = "kamn:did:agent:live-sdk";
const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
pub(crate) const REQUEST_AUTH_SCOPE_HEADER: &str = "x-kamn-authz-scope";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(crate) fn did(identifier: &str) -> AgentDid {
    AgentDid::parse(format!("kamn:did:agent:{identifier}")).expect("did should parse")
}

pub(crate) fn metadata(agent_type: &str, model: &str, capabilities: &[&str]) -> AgentMetadata {
    AgentMetadata {
        agent_type: agent_type.to_owned(),
        model_family: model.to_owned(),
        capabilities: capabilities
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

pub(crate) fn ensure_live_test_env() {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    std::env::set_var(LIVE_CHAIN_ID_ENV, CHAIN_ID);
    std::env::set_var(LIVE_CHAIN_VERSION_ENV, CHAIN_VERSION);
    std::env::set_var(LIVE_REQUESTER_DID_ENV, "kamn:did:agent:live-tester");
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) fn with_env_lock<T>(callback: impl FnOnce() -> T) -> T {
    let guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let output = callback();
    drop(guard);
    output
}

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn wait_for_server_ready(bind_addr: &str) {
    assert!(
        !bind_addr.trim().is_empty(),
        "server address must not be empty"
    );
    thread::sleep(Duration::from_millis(40));
}

pub(crate) fn deterministic_message_id(service_message_id: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in service_message_id.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}
