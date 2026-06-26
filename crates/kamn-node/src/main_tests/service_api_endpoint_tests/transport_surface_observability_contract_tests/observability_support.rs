use super::super::*;

pub(super) fn spawn_correlation_client(
    bind_addr: String,
    state_hash: &str,
) -> thread::JoinHandle<String> {
    let signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-correlation",
        41,
        state_hash,
        "{\"message\":\"structured-correlation\"}",
    );
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            "{\"message\":\"structured-correlation\"}",
            &[
                (
                    "X-KAMN-Sender-DID",
                    "kamn:did:agent:test-client-correlation",
                ),
                ("X-KAMN-Request-Nonce", "41"),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        )
    })
}

pub(super) fn assert_correlation_markers(captured_logs: &[String]) {
    let ingress_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.received\""))
        .expect("service api ingress should emit received marker");
    let outcome_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.outcome\""))
        .expect("service api ingress should emit outcome marker");
    assert_eq!(
        extract_json_string_field(ingress_line, "correlation_id"),
        extract_json_string_field(outcome_line, "correlation_id")
    );
    assert_eq!(
        extract_json_string_field(ingress_line, "method").as_deref(),
        Some("POST")
    );
    assert_eq!(
        extract_json_string_field(ingress_line, "path").as_deref(),
        Some("/v1/messages/send")
    );
    assert_eq!(
        extract_json_string_field(outcome_line, "status_code").as_deref(),
        Some("202")
    );
}
