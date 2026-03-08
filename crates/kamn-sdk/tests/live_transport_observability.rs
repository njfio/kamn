#[path = "support/live_transport_observability.rs"]
mod support;

use kamn_sdk::{LiveTransportKamnClient, SdkError};
use std::io::Write;
use std::thread;

use support::{parse_http_request, reserve_loopback_addr, wait_for_server_ready};

#[test]
fn spec_c10_live_transport_service_observability_routes_execute_network_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_observability_server(server_addr, false));
    wait_for_server_ready();

    let client = live_client(bind_addr.as_str());
    let health = client.service_health().expect("service_health should succeed");
    assert_eq!(health.status, "ok");
    assert_eq!(health.runtime_mode, "api");
    assert_eq!(health.role, "node");
    assert_eq!(health.observability_source, "service");
    assert_eq!(health.observability_health, "green");

    let metrics = client.service_metrics().expect("service_metrics should succeed");
    assert!(
        metrics.contains("kamn_service_api_health{runtime_mode=\"api\"} 1"),
        "metrics should include service health gauge"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "observability contract server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_service_health_rejects_malformed_payload() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_observability_server(server_addr, true));
    wait_for_server_ready();

    let client = live_client(bind_addr.as_str());
    assert_eq!(
        client.service_health(),
        Err(SdkError::TransportFailure(
            "service response missing required field"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "malformed health payload should satisfy request budget"
    );
}

fn live_client(endpoint: &str) -> LiveTransportKamnClient {
    let endpoint = format!("http://{endpoint}");
    LiveTransportKamnClient::connect(endpoint.as_str()).expect("live client should connect")
}

fn run_observability_server(bind_addr: String, malformed_health: bool) -> Result<(), String> {
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
    while served < 2 {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path, _, _) = parse_http_request(&mut stream)?;
                match (method.as_str(), path.as_str(), malformed_health) {
                    ("GET", "/healthz", false) => write_http_response(
                        &mut stream,
                        200,
                        r#"{"status":"ok","runtime_mode":"api","role":"node","observability_source":"service","observability_health":"green"}"#,
                    )?,
                    ("GET", "/healthz", true) => {
                        write_http_response(&mut stream, 200, r#"{"status":"ok"}"#)?
                    }
                    ("GET", "/metrics", false) => write_http_response(
                        &mut stream,
                        200,
                        "# HELP kamn_service_api_health service health\n# TYPE kamn_service_api_health gauge\nkamn_service_api_health{runtime_mode=\"api\"} 1\n",
                    )?,
                    _ => return Err(format!("unexpected route {method} {path}")),
                }
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

fn write_http_response(
    stream: &mut std::net::TcpStream,
    status: u16,
    body: &str,
) -> Result<(), String> {
    let status_text = match status {
        200 => "200 OK",
        _ => "500 Internal Server Error",
    };
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("response write failed: {error}"))
}
