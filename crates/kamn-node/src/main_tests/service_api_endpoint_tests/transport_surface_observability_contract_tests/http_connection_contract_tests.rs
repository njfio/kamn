use super::super::*;
use super::support::{assert_server_ok, build_transport_snapshot, spawn_transport_server};

#[test]
fn integration_service_api_endpoint_supports_keep_alive_requests_on_single_connection() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_transport_snapshot("127.0.0.1:34059");
    let server = spawn_transport_server(&snapshot, 2);
    let mut stream = TcpStream::connect(server.bind_addr.as_str()).expect("endpoint should accept");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");

    let request_one = format!(
        "GET /healthz HTTP/1.1\r\nHost: {}\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n",
        server.bind_addr
    );
    stream
        .write_all(request_one.as_bytes())
        .expect("first request should write");
    let first_response = read_single_http_response(&mut stream);
    assert!(first_response.contains("HTTP/1.1 200 OK"));
    assert!(first_response.contains("\"status\":\"ok\""));

    let request_two = format!(
        "GET /metrics HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
        server.bind_addr
    );
    stream
        .write_all(request_two.as_bytes())
        .expect("second request should write over keep-alive connection");
    let second_response = read_single_http_response(&mut stream);
    assert!(second_response.contains("HTTP/1.1 200 OK"));
    assert!(second_response
        .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1"));
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after keep-alive request budget",
    );
}
