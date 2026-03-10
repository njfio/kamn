use super::*;
#[test]
fn regression_http_transport_reuses_keep_alive_connection_pool_for_nonce_requests() {
    let (base_url, requests, server_handle) = spawn_keep_alive_multi_request_server(
        "{\"next_nonce\":42,\"account_id\":\"acc-42\"}".to_owned(),
        "HTTP/1.1 200 OK",
        2,
    );
    assert_nonce_pair(base_url.as_str());
    assert_keep_alive_server_metrics(server_handle, requests);
}

fn assert_nonce_pair(base_url: &str) {
    let nonce_request =
        KolmeApiNextNonceRequest::new("pub:key/keepalive").expect("request should build");
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let first = transport
        .fetch_next_nonce(base_url, "/get-next-nonce", &nonce_request)
        .expect("first nonce request should succeed");
    let second = transport
        .fetch_next_nonce(base_url, "/get-next-nonce", &nonce_request)
        .expect("second nonce request should succeed");
    assert_eq!(first.next_nonce, 42);
    assert_eq!(second.next_nonce, 42);
}

fn assert_keep_alive_server_metrics(
    server_handle: KeepAliveServerHandle,
    requests: KeepAliveRequestLog,
) {
    let (accepted_connections, handled_requests) = server_handle
        .join()
        .expect("keep-alive server thread should join");
    assert_eq!(handled_requests, 2);
    assert_eq!(accepted_connections, 1);
    let recorded_requests = requests.lock().expect("request log mutex should lock");
    assert_eq!(recorded_requests.len(), 2);
    assert!(recorded_requests.iter().all(|request| request.contains("Connection: keep-alive")));
}
