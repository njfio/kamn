use kamn_sdk::{AgentDid, SdkError, TcpSignedEnvelope, TcpTransportAdapter, TcpTransportConfig};
use std::collections::HashMap;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const DRIFT_FIXTURE: &str =
    include_str!("../../../fixtures/sdk_failover_reconnect/reconnect_drift_signatures.txt");

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

fn parse_fixture_signatures() -> HashMap<String, String> {
    let mut cases = HashMap::new();
    for raw_line in DRIFT_FIXTURE.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (scenario, signature) = match line.split_once('|') {
            Some(parts) => parts,
            None => panic!("invalid fixture line format, expected scenario|expected_signature"),
        };
        cases.insert(scenario.trim().to_owned(), signature.trim().to_owned());
    }
    cases
}

fn sender_config(addr: &str) -> TcpTransportConfig {
    let config = match TcpTransportConfig::new(addr) {
        Ok(value) => value,
        Err(error) => panic!("sender config failed: {error}"),
    };
    let config = match config.with_connect_retries(40) {
        Ok(value) => value,
        Err(error) => panic!("sender retry config failed: {error}"),
    };
    match config.with_retry_delay_millis(20) {
        Ok(value) => value,
        Err(error) => panic!("sender retry delay config failed: {error}"),
    }
}

fn send_raw_payload(addr: &str, payload: &str) {
    for _attempt in 0..60 {
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
fn unit_failover_reconnect_fixture_is_well_formed() {
    let signatures = parse_fixture_signatures();
    assert_eq!(signatures.len(), 3);
    assert_eq!(
        signatures
            .get("duplicate_nonce_same_route")
            .map(String::as_str),
        Some("tcp handshake replay detected")
    );
    assert_eq!(
        signatures
            .get("forged_handshake_signature")
            .map(String::as_str),
        Some("handshake.signature")
    );
    assert_eq!(
        signatures
            .get("missing_handshake_delimiter")
            .map(String::as_str),
        Some("missing handshake frame delimiter")
    );
}

#[test]
fn functional_primary_loss_reconnect_and_catchup_matrix_case() {
    let addr = free_addr();
    let listener_adapter = TcpTransportAdapter::new(match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    });
    let sender_adapter = TcpTransportAdapter::new(sender_config(addr.as_str()));

    let first = match TcpSignedEnvelope::new(
        did("sender-failover"),
        did("listener-failover"),
        1,
        "state:matrix",
        "primary-online",
    ) {
        Ok(value) => value,
        Err(error) => panic!("first envelope build failed: {error}"),
    };
    let second = match TcpSignedEnvelope::new(
        did("sender-failover"),
        did("listener-failover"),
        2,
        "state:matrix",
        "reconnect-catchup",
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

    let delayed_sender = sender_adapter.clone();
    let delayed_envelope = second.clone();
    let sender_thread = thread::spawn(move || delayed_sender.send(&delayed_envelope));
    thread::sleep(Duration::from_millis(120));

    let second_listener = listener_adapter.clone();
    let second_thread = thread::spawn(move || second_listener.listen_once());

    match sender_thread.join() {
        Ok(result) => {
            if let Err(error) = result {
                panic!("reconnect send failed: {error}");
            }
        }
        Err(_) => panic!("sender thread panicked"),
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
fn integration_three_process_failover_matrix_case() {
    let primary_addr = free_addr();
    let standby_addr = free_addr();

    let primary_listener =
        TcpTransportAdapter::new(match TcpTransportConfig::new(primary_addr.as_str()) {
            Ok(value) => value,
            Err(error) => panic!("primary listener config failed: {error}"),
        });
    let standby_listener =
        TcpTransportAdapter::new(match TcpTransportConfig::new(standby_addr.as_str()) {
            Ok(value) => value,
            Err(error) => panic!("standby listener config failed: {error}"),
        });
    let sender_primary = TcpTransportAdapter::new(sender_config(primary_addr.as_str()));
    let sender_standby = TcpTransportAdapter::new(sender_config(standby_addr.as_str()));

    let primary_envelope = match TcpSignedEnvelope::new(
        did("sender-three-process"),
        did("listener-three-process"),
        1,
        "state:three-process",
        "primary-path",
    ) {
        Ok(value) => value,
        Err(error) => panic!("primary envelope build failed: {error}"),
    };
    let standby_envelope = match TcpSignedEnvelope::new(
        did("sender-three-process"),
        did("listener-three-process"),
        2,
        "state:three-process",
        "standby-path",
    ) {
        Ok(value) => value,
        Err(error) => panic!("standby envelope build failed: {error}"),
    };

    let primary_thread = thread::spawn(move || primary_listener.listen_once());
    thread::sleep(Duration::from_millis(30));
    if let Err(error) = sender_primary.send(&primary_envelope) {
        panic!("primary send failed: {error}");
    }
    let primary_received = match primary_thread.join() {
        Ok(result) => match result {
            Ok(value) => value,
            Err(error) => panic!("primary listen failed: {error}"),
        },
        Err(_) => panic!("primary listener thread panicked"),
    };
    assert_eq!(primary_received.envelope, primary_envelope);

    let standby_thread = thread::spawn(move || standby_listener.listen_once());
    thread::sleep(Duration::from_millis(30));
    if let Err(error) = sender_standby.send(&standby_envelope) {
        panic!("standby send failed: {error}");
    }
    let standby_received = match standby_thread.join() {
        Ok(result) => match result {
            Ok(value) => value,
            Err(error) => panic!("standby listen failed: {error}"),
        },
        Err(_) => panic!("standby listener thread panicked"),
    };
    assert_eq!(standby_received.envelope, standby_envelope);
}

#[test]
fn regression_reconnect_drift_signature_fixture_contract() {
    // Regression: #824
    let signatures = parse_fixture_signatures();
    let replay_signature = match signatures.get("duplicate_nonce_same_route") {
        Some(value) => value,
        None => panic!("duplicate_nonce_same_route fixture missing"),
    };
    let forged_signature = match signatures.get("forged_handshake_signature") {
        Some(value) => value,
        None => panic!("forged_handshake_signature fixture missing"),
    };

    let replay_addr = free_addr();
    let replay_listener =
        TcpTransportAdapter::new(match TcpTransportConfig::new(replay_addr.as_str()) {
            Ok(value) => value,
            Err(error) => panic!("replay listener config failed: {error}"),
        });
    let replay_sender = TcpTransportAdapter::new(sender_config(replay_addr.as_str()));

    let first = match TcpSignedEnvelope::new(
        did("sender-regression-replay"),
        did("listener-regression-replay"),
        8,
        "state:regression",
        "replay-initial",
    ) {
        Ok(value) => value,
        Err(error) => panic!("first replay envelope build failed: {error}"),
    };
    let replayed = match TcpSignedEnvelope::new(
        did("sender-regression-replay"),
        did("listener-regression-replay"),
        8,
        "state:regression",
        "replay-duplicate",
    ) {
        Ok(value) => value,
        Err(error) => panic!("replay envelope build failed: {error}"),
    };

    let first_listener = replay_listener.clone();
    let first_thread = thread::spawn(move || first_listener.listen_once());
    thread::sleep(Duration::from_millis(30));
    if let Err(error) = replay_sender.send(&first) {
        panic!("first replay send failed: {error}");
    }
    match first_thread.join() {
        Ok(result) => {
            if let Err(error) = result {
                panic!("first replay listen failed: {error}");
            }
        }
        Err(_) => panic!("first replay listener thread panicked"),
    }

    let replay_listener_thread = {
        let listener = replay_listener.clone();
        thread::spawn(move || listener.listen_once())
    };
    thread::sleep(Duration::from_millis(30));
    if let Err(error) = replay_sender.send(&replayed) {
        panic!("replay send failed: {error}");
    }
    let replay_result = match replay_listener_thread.join() {
        Ok(result) => result,
        Err(_) => panic!("replay listener thread panicked"),
    };
    match replay_result {
        Err(SdkError::Conflict(reason)) => assert_eq!(reason, replay_signature),
        other => panic!("expected replay conflict, got: {other:?}"),
    }

    let forged_addr = free_addr();
    let forged_listener =
        TcpTransportAdapter::new(match TcpTransportConfig::new(forged_addr.as_str()) {
            Ok(value) => value,
            Err(error) => panic!("forged listener config failed: {error}"),
        });
    let forged_envelope = match TcpSignedEnvelope::new(
        did("sender-regression-forged"),
        did("listener-regression-forged"),
        5,
        "state:regression",
        "forged-frame",
    ) {
        Ok(value) => value,
        Err(error) => panic!("forged envelope build failed: {error}"),
    };
    let forged_payload = format!(
        "frame=handshake\n\
version=1\n\
profile=ed25519:baseline-v1\n\
from={}\n\
to={}\n\
nonce={}\n\
signature=sig:ed25519:baseline-v1:forged-signature\n\n{}",
        forged_envelope.from,
        forged_envelope.to,
        forged_envelope.nonce,
        forged_envelope.to_wire_payload()
    );

    let forged_listener_thread = thread::spawn(move || forged_listener.listen_once());
    thread::sleep(Duration::from_millis(30));
    send_raw_payload(forged_addr.as_str(), forged_payload.as_str());

    let forged_result = match forged_listener_thread.join() {
        Ok(result) => result,
        Err(_) => panic!("forged listener thread panicked"),
    };
    match forged_result {
        Err(SdkError::InvalidInput { field, .. }) => assert_eq!(field, forged_signature),
        other => panic!("expected forged handshake invalid input, got: {other:?}"),
    }
}

#[test]
fn performance_tcp_failover_reconnect_matrix_fast_lane_budget() {
    let started = Instant::now();
    let addr = free_addr();
    let listener_adapter = TcpTransportAdapter::new(match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    });
    let sender_adapter = TcpTransportAdapter::new(sender_config(addr.as_str()));

    for nonce in 1..=12 {
        let sender = sender_adapter.clone();
        let envelope = match TcpSignedEnvelope::new(
            did("sender-perf-failover"),
            did("listener-perf-failover"),
            nonce,
            "state:perf-failover",
            format!("payload-{nonce}"),
        ) {
            Ok(value) => value,
            Err(error) => panic!("perf envelope build failed: {error}"),
        };

        let sender_thread = thread::spawn(move || sender.send(&envelope));
        thread::sleep(Duration::from_millis(15));
        let listener = listener_adapter.clone();
        let listener_thread = thread::spawn(move || listener.listen_once());

        match sender_thread.join() {
            Ok(result) => {
                if let Err(error) = result {
                    panic!("perf send failed at nonce {nonce}: {error}");
                }
            }
            Err(_) => panic!("perf sender thread panicked"),
        }
        match listener_thread.join() {
            Ok(result) => {
                if let Err(error) = result {
                    panic!("perf listen failed at nonce {nonce}: {error}");
                }
            }
            Err(_) => panic!("perf listener thread panicked"),
        }
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 1200,
        "tcp failover reconnect fast lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled tcp failover reconnect deep lane"]
fn performance_tcp_failover_reconnect_matrix_deep_lane() {
    let addr = free_addr();
    let listener_adapter = TcpTransportAdapter::new(match TcpTransportConfig::new(addr.as_str()) {
        Ok(value) => value,
        Err(error) => panic!("listener config failed: {error}"),
    });
    let sender_adapter = TcpTransportAdapter::new(sender_config(addr.as_str()));

    for nonce in 1..=250 {
        let listener = listener_adapter.clone();
        let listener_thread = thread::spawn(move || listener.listen_once());
        thread::sleep(Duration::from_millis(2));

        let envelope = match TcpSignedEnvelope::new(
            did("sender-deep-failover"),
            did("listener-deep-failover"),
            nonce,
            "state:deep-failover",
            format!("deep-payload-{nonce}"),
        ) {
            Ok(value) => value,
            Err(error) => panic!("deep envelope build failed: {error}"),
        };
        if let Err(error) = sender_adapter.send(&envelope) {
            panic!("deep send failed at nonce {nonce}: {error}");
        }

        match listener_thread.join() {
            Ok(result) => {
                if let Err(error) = result {
                    panic!("deep listen failed at nonce {nonce}: {error}");
                }
            }
            Err(_) => panic!("deep listener thread panicked"),
        }
    }
}
