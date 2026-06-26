use super::*;

pub(crate) fn broadcast_request(
    message: &str,
    signature: &str,
    recovery_id: u8,
) -> KolmeApiBroadcastRequest {
    KolmeApiBroadcastRequest::new(message, signature, recovery_id)
        .expect("broadcast request should build")
}

pub(crate) fn new_broadcast_transport() -> KolmeRuntimeCommitHttpTransport {
    KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build")
}

pub(crate) fn broadcast_server(txhash: &str, matcher: impl Fn(String) + Send + 'static) -> String {
    spawn_single_request_server(
        format!("{{\"txhash\":\"{txhash}\"}}"),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(request.contains("Content-Type: application/json"));
            matcher(request);
        },
    )
}

pub(crate) fn submit_broadcast(
    base_url: &str,
    path: &str,
    request: &KolmeApiBroadcastRequest,
    idempotency_key: &str,
) -> String {
    new_broadcast_transport()
        .submit_broadcast_request(base_url, path, request, idempotency_key)
        .expect("broadcast helper should succeed")
        .txhash
}
