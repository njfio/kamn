#[path = "support/live_transport_events.rs"]
mod support;

use kamn_sdk::{KamnServiceEvents, LiveTransportKamnClient, SdkError, ServiceEventSnapshot};
use std::io::Write;
use std::thread;

use support::{parse_http_request, reserve_loopback_addr, wait_for_server_ready};

#[test]
fn spec_c11_live_transport_service_events_route_executes_network_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_event_server(server_addr, false));
    wait_for_server_ready();

    let client = live_client(bind_addr.as_str());
    let event = read_service_event(&client).expect("service event should succeed");
    assert_eq!(event.event, "state-transition");
    assert_eq!(event.runtime_mode, "api");
    assert_eq!(event.role, "processor");
    assert_eq!(event.sequence, 1);

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "events contract server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_service_event_rejects_malformed_payload() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_event_server(server_addr, true));
    wait_for_server_ready();

    let client = live_client(bind_addr.as_str());
    assert_eq!(
        read_service_event(&client),
        Err(SdkError::TransportFailure(
            "service response missing required field"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "malformed event payload should satisfy request budget"
    );
}

#[test]
fn regression_in_memory_client_does_not_implement_service_events_trait() {
    let memory_source = include_str!("../src/memory.rs");
    assert!(
        !memory_source.contains("impl KamnServiceEvents for InMemoryKamnClient"),
        "in-memory client should stay outside the service events trait surface"
    );
}

fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}

fn read_service_event<T: KamnServiceEvents>(client: &T) -> Result<ServiceEventSnapshot, SdkError> {
    client.read_service_event()
}

fn run_event_server(bind_addr: String, malformed_event: bool) -> Result<(), String> {
    use std::net::TcpListener;
    use std::thread;
    use std::time::{Duration, Instant};

    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("nonblocking setup failed: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    while served < 1 {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path, _, headers) = parse_http_request(&mut stream)?;
                if method != "GET" || path != "/v1/events/ws" {
                    return Err(format!("unexpected route {method} {path}"));
                }
                let upgrade = headers.get("upgrade").cloned().unwrap_or_default();
                let connection = headers.get("connection").cloned().unwrap_or_default();
                let websocket_key = headers
                    .get("sec-websocket-key")
                    .cloned()
                    .unwrap_or_default();
                let version = headers
                    .get("sec-websocket-version")
                    .cloned()
                    .unwrap_or_default();
                if !upgrade.eq_ignore_ascii_case("websocket")
                    || !connection.to_ascii_lowercase().contains("upgrade")
                    || websocket_key.trim().is_empty()
                    || version.trim() != "13"
                {
                    return Err("websocket upgrade headers missing or invalid".to_owned());
                }
                let payload = if malformed_event {
                    r#"{"event":"state-transition","runtime_mode":"api","role":"processor"}"#
                } else {
                    r#"{"event":"state-transition","runtime_mode":"api","role":"processor","sequence":1}"#
                };
                write_websocket_upgrade_response(&mut stream, payload)?;
                served = served.saturating_add(1);
            }
            Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock) => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("accept failed: {error}")),
        }
    }
    Ok(())
}

fn write_websocket_upgrade_response(
    stream: &mut std::net::TcpStream,
    payload: &str,
) -> Result<(), String> {
    let handshake = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: kamn-test-accept\r\nX-KAMN-WebSocket-Contract: v1\r\n\r\n";
    stream
        .write_all(handshake.as_bytes())
        .map_err(|error| format!("websocket handshake write failed: {error}"))?;
    let payload_bytes = payload.as_bytes();
    let mut frame = Vec::with_capacity(payload_bytes.len() + 4);
    frame.push(0x81);
    if payload_bytes.len() <= 125 {
        frame.push(payload_bytes.len() as u8);
    } else {
        frame.push(126);
        frame.extend_from_slice(&(payload_bytes.len() as u16).to_be_bytes());
    }
    frame.extend_from_slice(payload_bytes);
    stream
        .write_all(frame.as_slice())
        .map_err(|error| format!("websocket frame write failed: {error}"))
}
