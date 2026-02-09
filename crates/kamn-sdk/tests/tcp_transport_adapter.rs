use kamn_sdk::{
    signature_for_fields, AgentDid, SdkError, TcpSignedEnvelope, TcpTransportAdapter,
    TcpTransportConfig,
};
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

fn did(value: &str) -> AgentDid {
    match AgentDid::parse(format!("kamn:did:agent:{value}")) {
        Ok(parsed) => parsed,
        Err(error) => panic!("did parse failed: {error}"),
    }
}

fn free_addr() -> String {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(bound) => bound,
        Err(error) => panic!("failed to allocate free tcp address: {error}"),
    };
    let addr = match listener.local_addr() {
        Ok(local) => local,
        Err(error) => panic!("failed to read allocated local address: {error}"),
    };
    addr.to_string()
}

fn send_raw_payload(addr: &str, payload: &str) {
    for _attempt in 0..40 {
        match TcpStream::connect(addr) {
            Ok(mut stream) => {
                if let Err(error) = stream.write_all(payload.as_bytes()) {
                    panic!("failed to write raw payload: {error}");
                }
                if let Err(error) = stream.flush() {
                    panic!("failed to flush raw payload: {error}");
                }
                if let Err(error) = stream.shutdown(Shutdown::Write) {
                    panic!("failed to shutdown raw payload stream: {error}");
                }
                return;
            }
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    }
    panic!("failed to connect raw payload sender to {addr}");
}

#[test]
fn unit_tcp_envelope_roundtrip_is_deterministic() {
    let envelope = match TcpSignedEnvelope::new(
        did("sender-1"),
        did("listener-1"),
        7,
        "state:runtime-7",
        "hello-runtime",
    ) {
        Ok(value) => value,
        Err(error) => panic!("envelope build failed: {error}"),
    };

    let wire = envelope.to_wire_payload();
    let parsed = match TcpSignedEnvelope::parse_wire_payload(&wire) {
        Ok(value) => value,
        Err(error) => panic!("envelope parse failed: {error}"),
    };

    assert_eq!(parsed, envelope);
}

#[test]
fn unit_tcp_envelope_rejects_duplicate_keys() {
    let payload = "from=kamn:did:agent:sender-1\nfrom=kamn:did:agent:sender-1\nto=kamn:did:agent:listener-1\nnonce=1\nstate_hash=state:dup\nbody=dup\nsignature=sig:ed25519:baseline-v1:kamn:did:agent:sender-1:1:state:dup:3\n";

    assert_eq!(
        TcpSignedEnvelope::parse_wire_payload(payload),
        Err(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "duplicate key: from",
        })
    );
}

#[test]
fn functional_tcp_adapter_relays_signed_envelope_between_two_processes() {
    let addr = free_addr();

    let listener_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    };
    let sender_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("sender config failed: {error}"),
    };

    let listener_adapter = TcpTransportAdapter::new(listener_config);
    let sender_adapter = TcpTransportAdapter::new(sender_config);

    let expected_envelope = match TcpSignedEnvelope::new(
        did("sender-func"),
        did("listener-func"),
        11,
        "state:functional",
        "functional-envelope",
    ) {
        Ok(value) => value,
        Err(error) => panic!("envelope build failed: {error}"),
    };

    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));

    if let Err(error) = sender_adapter.send(&expected_envelope) {
        panic!("sender adapter failed to send envelope: {error}");
    }

    let received = match listener_thread.join() {
        Ok(result) => match result {
            Ok(value) => value,
            Err(error) => panic!("listener adapter failed: {error}"),
        },
        Err(_) => panic!("listener thread panicked"),
    };

    assert_eq!(received.envelope, expected_envelope);
    assert!(received.peer_addr.starts_with("127.0.0.1:"));
}

#[test]
fn integration_tcp_adapter_rejects_oversized_wire_payload() {
    let addr = free_addr();

    let listener_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => match value.with_max_wire_bytes(16) {
            Ok(config) => config,
            Err(error) => panic!("listener max-wire config failed: {error}"),
        },
        Err(error) => panic!("listener config failed: {error}"),
    };
    let sender_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("sender config failed: {error}"),
    };

    let listener_adapter = TcpTransportAdapter::new(listener_config);
    let sender_adapter = TcpTransportAdapter::new(sender_config);

    let large_body = "x".repeat(64);
    let envelope = match TcpSignedEnvelope::new(
        did("sender-large"),
        did("listener-large"),
        1,
        "state:large",
        large_body,
    ) {
        Ok(value) => value,
        Err(error) => panic!("envelope build failed: {error}"),
    };

    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));

    if let Err(error) = sender_adapter.send(&envelope) {
        panic!("sender adapter failed: {error}");
    }

    let listener_result = match listener_thread.join() {
        Ok(result) => result,
        Err(_) => panic!("listener thread panicked"),
    };

    assert_eq!(
        listener_result,
        Err(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "exceeds max wire bytes",
        })
    );
}

#[test]
fn functional_tcp_adapter_reconnect_preserves_nonce_replay_guard_state() {
    let addr = free_addr();

    let listener_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    };
    let sender_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("sender config failed: {error}"),
    };

    let listener_adapter = TcpTransportAdapter::new(listener_config);
    let sender_adapter = TcpTransportAdapter::new(sender_config);

    let first = match TcpSignedEnvelope::new(
        did("sender-reconnect"),
        did("listener-reconnect"),
        1,
        "state:reconnect",
        "first-connect",
    ) {
        Ok(value) => value,
        Err(error) => panic!("first envelope build failed: {error}"),
    };
    let second = match TcpSignedEnvelope::new(
        did("sender-reconnect"),
        did("listener-reconnect"),
        2,
        "state:reconnect",
        "second-connect",
    ) {
        Ok(value) => value,
        Err(error) => panic!("second envelope build failed: {error}"),
    };

    let first_listener = listener_adapter.clone();
    let first_thread = thread::spawn(move || first_listener.listen_once());
    thread::sleep(Duration::from_millis(30));

    if let Err(error) = sender_adapter.send(&first) {
        panic!("first send failed: {error}");
    }
    let first_received = match first_thread.join() {
        Ok(result) => match result {
            Ok(value) => value,
            Err(error) => panic!("first listen failed: {error}"),
        },
        Err(_) => panic!("first listener thread panicked"),
    };
    assert_eq!(first_received.envelope, first);

    let second_listener = listener_adapter.clone();
    let second_thread = thread::spawn(move || second_listener.listen_once());
    thread::sleep(Duration::from_millis(30));

    if let Err(error) = sender_adapter.send(&second) {
        panic!("second send failed: {error}");
    }
    let second_received = match second_thread.join() {
        Ok(result) => match result {
            Ok(value) => value,
            Err(error) => panic!("second listen failed: {error}"),
        },
        Err(_) => panic!("second listener thread panicked"),
    };
    assert_eq!(second_received.envelope, second);
}

#[test]
fn integration_tcp_adapter_replay_nonce_is_rejected_across_reconnect() {
    let addr = free_addr();

    let listener_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    };
    let sender_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("sender config failed: {error}"),
    };

    let listener_adapter = TcpTransportAdapter::new(listener_config);
    let sender_adapter = TcpTransportAdapter::new(sender_config);

    let first = match TcpSignedEnvelope::new(
        did("sender-replay"),
        did("listener-replay"),
        9,
        "state:replay",
        "nonce-9-initial",
    ) {
        Ok(value) => value,
        Err(error) => panic!("first envelope build failed: {error}"),
    };
    let replayed = match TcpSignedEnvelope::new(
        did("sender-replay"),
        did("listener-replay"),
        9,
        "state:replay",
        "nonce-9-replayed",
    ) {
        Ok(value) => value,
        Err(error) => panic!("replayed envelope build failed: {error}"),
    };

    let first_listener = listener_adapter.clone();
    let first_thread = thread::spawn(move || first_listener.listen_once());
    thread::sleep(Duration::from_millis(30));
    if let Err(error) = sender_adapter.send(&first) {
        panic!("first send failed: {error}");
    }
    match first_thread.join() {
        Ok(result) => {
            if let Err(error) = result {
                panic!("first listen failed: {error}");
            }
        }
        Err(_) => panic!("first listener thread panicked"),
    }

    let replay_listener = listener_adapter.clone();
    let replay_thread = thread::spawn(move || replay_listener.listen_once());
    thread::sleep(Duration::from_millis(30));
    if let Err(error) = sender_adapter.send(&replayed) {
        panic!("replay send failed: {error}");
    }
    let replay_result = match replay_thread.join() {
        Ok(result) => result,
        Err(_) => panic!("replay listener thread panicked"),
    };
    assert_eq!(
        replay_result,
        Err(SdkError::Conflict("tcp handshake replay detected"))
    );
}

#[test]
fn regression_tampered_tcp_envelope_signature_is_rejected() {
    // Regression: #822
    let signature = signature_for_fields(
        "kamn:did:agent:sender-regression",
        1,
        "state:regression",
        "expected-body",
    );
    let tampered_payload = format!(
        "from=kamn:did:agent:sender-regression\n\
to=kamn:did:agent:listener-regression\n\
nonce=1\n\
state_hash=state:regression\n\
body=tampered-body-extended\n\
signature={signature}\n"
    );

    assert_eq!(
        TcpSignedEnvelope::parse_wire_payload(tampered_payload.as_str()),
        Err(SdkError::InvalidInput {
            field: "signature",
            reason: "does not match deterministic envelope fields",
        })
    );
}

#[test]
fn regression_forged_handshake_frame_is_rejected() {
    // Regression: #823
    let addr = free_addr();
    let listener_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    };
    let listener_adapter = TcpTransportAdapter::new(listener_config);

    let envelope = match TcpSignedEnvelope::new(
        did("sender-forged"),
        did("listener-forged"),
        5,
        "state:forged",
        "forged-handshake-frame",
    ) {
        Ok(value) => value,
        Err(error) => panic!("envelope build failed: {error}"),
    };

    let forged_payload = format!(
        "frame=handshake\n\
version=1\n\
profile=ed25519:baseline-v1\n\
from={}\n\
to={}\n\
nonce={}\n\
signature=sig:ed25519:baseline-v1:forged-signature\n\n{}",
        envelope.from,
        envelope.to,
        envelope.nonce,
        envelope.to_wire_payload()
    );

    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));
    send_raw_payload(addr.as_str(), forged_payload.as_str());

    let listener_result = match listener_thread.join() {
        Ok(result) => result,
        Err(_) => panic!("listener thread panicked"),
    };
    assert_eq!(
        listener_result,
        Err(SdkError::InvalidInput {
            field: "handshake.signature",
            reason: "does not match envelope signature",
        })
    );
}

#[test]
fn performance_tcp_adapter_local_relay_contract_stays_within_budget() {
    let addr = free_addr();
    let listener_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    };
    let sender_config = match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("sender config failed: {error}"),
    };

    let listener_adapter = TcpTransportAdapter::new(listener_config);
    let sender_adapter = TcpTransportAdapter::new(sender_config);

    let envelope = match TcpSignedEnvelope::new(
        did("sender-perf"),
        did("listener-perf"),
        1,
        "state:perf",
        "perf-envelope",
    ) {
        Ok(value) => value,
        Err(error) => panic!("envelope build failed: {error}"),
    };

    let started = Instant::now();
    let listener_thread = thread::spawn(move || listener_adapter.listen_once());
    thread::sleep(Duration::from_millis(30));

    if let Err(error) = sender_adapter.send(&envelope) {
        panic!("sender adapter failed: {error}");
    }

    let received = match listener_thread.join() {
        Ok(result) => match result {
            Ok(value) => value,
            Err(error) => panic!("listener failed: {error}"),
        },
        Err(_) => panic!("listener thread panicked"),
    };
    assert_eq!(received.envelope, envelope);

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 500,
        "tcp adapter relay contract lane exceeded budget: {elapsed_millis}ms"
    );
}
