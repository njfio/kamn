use super::support::*;
use super::*;

#[path = "stream_runtime_contract_tests/stream_server_contract_tests.rs"]
mod stream_server_contract_tests;

fn assert_stream_payload(snapshot: &RuntimeObservabilitySnapshot) {
    let stream = render_observability_endpoint_response(snapshot, "/metrics.stream");
    assert_eq!(stream.status_code, 200);
    assert_eq!(stream.content_type, "application/x-ndjson");
    assert!(stream
        .body
        .contains("\"schema_version\":\"kamn.runtime.observability.stream.v1\""));
    assert!(stream.body.ends_with('\n'));
}

#[test]
fn functional_observability_endpoint_renders_stream_payload() {
    assert_stream_payload(&daemon_observability_snapshot());
}

#[test]
fn integration_runtime_observability_endpoint_supports_stream_reconnect_churn_sequence() {
    let snapshot = sample_observability_snapshot();
    let (bind_addr, server) = spawn_observability_server(&snapshot, 4, 2_000);
    assert!(send_http_get(bind_addr.as_str(), "/metrics.stream").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/metrics.stream").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}
