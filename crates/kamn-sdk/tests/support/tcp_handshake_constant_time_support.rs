use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use kamn_sdk::{AgentDid, TcpSignedEnvelope};
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

pub const TEST_TCP_SIGNING_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
pub const TEST_TCP_SIGNING_PRIVATE_KEY_HEX_ALT: &str =
    "094cf4e1f3d974bbf3e72233e2c2937e8fdb094740e0f017e010aa47ac1201ac";

pub fn did(value: &str, private_key_hex: &str) -> AgentDid {
    let signer_public_key = signer_public_key_hex_for_private_key(private_key_hex);
    AgentDid::with_public_key_hex_binding(value, signer_public_key.as_str())
        .unwrap_or_else(|error| panic!("bound did parse failed: {error}"))
}

pub fn free_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|error| {
        panic!("failed to allocate free tcp address: {error}");
    });
    listener
        .local_addr()
        .unwrap_or_else(|error| panic!("failed to read allocated local address: {error}"))
        .to_string()
}

pub fn send_raw_payload(addr: &str, payload: &str) {
    for _attempt in 0..40 {
        match TcpStream::connect(addr) {
            Ok(mut stream) => {
                stream
                    .write_all(payload.as_bytes())
                    .unwrap_or_else(|error| panic!("failed to write raw payload: {error}"));
                stream
                    .flush()
                    .unwrap_or_else(|error| panic!("failed to flush raw payload: {error}"));
                stream.shutdown(Shutdown::Write).unwrap_or_else(|error| {
                    panic!("failed to shutdown raw payload stream: {error}");
                });
                return;
            }
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    }
    panic!("failed to connect raw payload sender to {addr}");
}

pub fn signer_public_key_hex_for_private_key(private_key_hex: &str) -> String {
    service_auth_public_key_hex_from_private_key_hex(private_key_hex).unwrap_or_else(|error| {
        panic!("failed to derive tcp signer public key for {private_key_hex}: {error}")
    })
}

pub fn handshake_signature(envelope: &TcpSignedEnvelope, signer_private_key_hex: &str) -> String {
    service_auth_sign_with_private_key_hex(
        envelope.from.as_str(),
        envelope.nonce,
        envelope.state_hash.as_str(),
        envelope.body.as_str(),
        signer_private_key_hex,
    )
    .unwrap_or_else(|error| panic!("handshake signature failed: {error}"))
}

pub fn handshake_payload(
    envelope: &TcpSignedEnvelope,
    signer_public_key: &str,
    signature: &str,
) -> String {
    format!(
        "frame=handshake\nversion=1\nprofile=secp256k1:baseline-v2\nfrom={}\nto={}\nnonce={}\nsigner_public_key={}\nsignature={}\n\n{}",
        envelope.from,
        envelope.to,
        envelope.nonce,
        signer_public_key,
        signature,
        envelope.to_wire_payload()
    )
}
