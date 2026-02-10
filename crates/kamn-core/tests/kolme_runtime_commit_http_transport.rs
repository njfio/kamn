use kamn_core::{
    KolmeCommitReceiptFinality, KolmeRuntimeCommitFinalityChecker, KolmeRuntimeCommitHttpTransport,
    KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitProvider, KolmeRuntimeCommitProviderError,
    KolmeRuntimeCommitProviderOutcome,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");

    loop {
        let read_count = stream
            .read(&mut chunk)
            .expect("request bytes should be readable");
        if read_count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_count]);

        if header_end.is_none() {
            header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|pos| pos + 4);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&buffer[..end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        if name.eq_ignore_ascii_case("Content-Length") {
                            return value.trim().parse::<usize>().ok();
                        }
                        None
                    })
                    .unwrap_or(0);
                expected_total = Some(end + content_length);
            }
        }

        if let Some(total) = expected_total {
            if buffer.len() >= total {
                break;
            }
        }
    }

    String::from_utf8(buffer).expect("request should be valid utf-8")
}

fn spawn_single_request_server(
    response_body: String,
    status_line: &str,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    let status_line = status_line.to_owned();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("response should write");
    });
    format!("http://{addr}")
}

fn spawn_server_with_raw_response(
    raw_response: String,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);
        stream
            .write_all(raw_response.as_bytes())
            .expect("response should write");
    });
    format!("http://{addr}")
}

#[test]
fn integration_http_transport_submit_and_response_mapping() {
    let wire_payload = "operation_id=op-1\nstate_root=state-1\n";
    let idempotency_key = "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1";
    let response_body =
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
            assert!(request.contains("X-Idempotency-Key: "));
            assert!(request.ends_with(wire_payload));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:1");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn integration_http_transport_finality_query_and_response_mapping() {
    let commit_id = "commit:id/with space";
    let response_body =
        "{\"provider\":\"kolme-local\",\"commit_id\":\"commit:id/with space\",\"finality\":\"final\"}";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains(
                "GET /runtime-commit/status?commit_id=commit%3Aid%2Fwith%20space HTTP/1.1"
            ));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        base_url.as_str(),
        "/runtime-commit/status",
        transport,
    )
    .expect("checker should build");

    let receipt = checker
        .check_commit_finality(commit_id)
        .expect("finality check should succeed");
    assert_eq!(receipt.provider, "kolme-local");
    assert_eq!(receipt.commit_id, commit_id);
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_http_transport_timeout_maps_to_provider_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("connection should be accepted");
        thread::sleep(Duration::from_secs(2));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        format!("http://{addr}").as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Timeout)
    );
}

#[test]
fn regression_http_transport_rejects_invalid_port_before_network_io() {
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:abc",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "base_url port is invalid".to_owned(),
        })
    );
}

#[test]
fn regression_http_transport_fails_closed_on_content_length_mismatch() {
    let body = "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n";
    let declared_length = body.len() + 9;
    let raw_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n{body}"
    );

    let base_url = spawn_server_with_raw_response(raw_response, |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!(
                "http response content-length mismatch: declared {declared_length}, observed {}",
                body.len()
            ),
        })
    );
}
