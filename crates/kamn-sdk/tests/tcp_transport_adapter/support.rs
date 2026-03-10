pub(crate) use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
pub(crate) use kamn_sdk::{
    signature_for_fields, AgentDid, SdkError, TcpSignedEnvelope, TcpTransportAdapter,
    TcpTransportConfig,
};
pub(crate) use std::io::Write;
pub(crate) use std::net::{Shutdown, TcpListener, TcpStream};
pub(crate) use std::thread;
pub(crate) use std::time::{Duration, Instant};

pub(crate) const TEST_TCP_SIGNING_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
pub(crate) const TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT: &str =
    "094cf4e1f3d974bbf3e72233e2c2937e8fdb094740e0f017e010aa47ac1201ac";

pub(crate) fn did_unbound(value: &str) -> AgentDid {
    AgentDid::parse(format!("kamn:did:agent:{value}"))
        .unwrap_or_else(|error| panic!("did parse failed: {error}"))
}

pub(crate) fn did(value: &str) -> AgentDid {
    let signer_public_key = signer_public_key_hex_for_private_key(TEST_TCP_SIGNING_PRIVATE_KEY_HEX);
    AgentDid::with_public_key_hex_binding(value, signer_public_key.as_str())
        .unwrap_or_else(|error| panic!("bound did parse failed: {error}"))
}

pub(crate) fn free_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .unwrap_or_else(|error| panic!("failed to allocate free tcp address: {error}"));
    listener
        .local_addr()
        .unwrap_or_else(|error| panic!("failed to read allocated local address: {error}"))
        .to_string()
}

pub(crate) fn send_raw_payload(addr: &str, payload: &str) {
    for _attempt in 0..40 {
        if try_send_raw_payload(addr, payload) {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("failed to connect raw payload sender to {addr}");
}

fn try_send_raw_payload(addr: &str, payload: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect(addr) else {
        return false;
    };
    stream
        .write_all(payload.as_bytes())
        .unwrap_or_else(|error| panic!("failed to write raw payload: {error}"));
    stream
        .flush()
        .unwrap_or_else(|error| panic!("failed to flush raw payload: {error}"));
    stream
        .shutdown(Shutdown::Write)
        .unwrap_or_else(|error| panic!("failed to shutdown raw payload stream: {error}"));
    true
}

pub(crate) fn signer_public_key_hex() -> String {
    signer_public_key_hex_for_private_key(TEST_TCP_SIGNING_PRIVATE_KEY_HEX)
}

pub(crate) fn signer_public_key_hex_for_private_key(private_key_hex: &str) -> String {
    service_auth_public_key_hex_from_private_key_hex(private_key_hex).unwrap_or_else(|error| {
        panic!("failed to derive tcp signer public key for {private_key_hex}: {error}")
    })
}

pub(crate) fn build_envelope(
    from: AgentDid,
    to: AgentDid,
    nonce: u64,
    state_hash: &str,
    body: &str,
) -> TcpSignedEnvelope {
    TcpSignedEnvelope::new(
        from,
        to,
        nonce,
        state_hash,
        body,
        TEST_TCP_SIGNING_PRIVATE_KEY_HEX,
    )
    .unwrap_or_else(|error| panic!("envelope build failed: {error}"))
}

pub(crate) fn adapter_pair(addr: &str) -> (TcpTransportAdapter, TcpTransportAdapter) {
    let listener = TcpTransportAdapter::new(tcp_config(addr));
    let sender = TcpTransportAdapter::new(tcp_config(addr));
    (listener, sender)
}

pub(crate) fn limited_listener_sender_pair(
    addr: &str,
    max_wire_bytes: usize,
) -> (TcpTransportAdapter, TcpTransportAdapter) {
    let listener = TcpTransportAdapter::new(tcp_config(addr).with_max_wire_bytes(max_wire_bytes).expect("listener max-wire config failed"));
    let sender = TcpTransportAdapter::new(tcp_config(addr));
    (listener, sender)
}

fn tcp_config(addr: &str) -> TcpTransportConfig {
    TcpTransportConfig::new(addr)
        .unwrap_or_else(|error| panic!("tcp config failed: {error}"))
}

pub(crate) fn listen_once_in_thread(
    adapter: TcpTransportAdapter,
) -> thread::JoinHandle<Result<kamn_sdk::TcpReceivedEnvelope, SdkError>> {
    thread::spawn(move || adapter.listen_once())
}

pub(crate) fn join_listener(
    handle: thread::JoinHandle<Result<kamn_sdk::TcpReceivedEnvelope, SdkError>>,
) -> Result<kamn_sdk::TcpReceivedEnvelope, SdkError> {
    handle.join().unwrap_or_else(|_| panic!("listener thread panicked"))
}

pub(crate) fn wait_for_listener() {
    thread::sleep(Duration::from_millis(30));
}
