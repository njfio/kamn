use super::super::*;

pub(super) struct AsyncTransportBurstResult {
    pub(super) health: Result<String, String>,
    pub(super) metrics: Result<String, String>,
    pub(super) send_one: Result<String, String>,
    pub(super) send_two: Result<String, String>,
}

pub(super) fn run_async_transport_burst(
    bind_addr: &str,
    state_hash: &str,
) -> AsyncTransportBurstResult {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("async runtime should initialize");
    runtime.block_on(async move {
        let one = async_signed_send(
            bind_addr,
            "kamn:did:agent:async-http-client-1",
            900,
            state_hash,
            "{\"message\":\"async-route-1\"}",
        );
        let two = async_signed_send(
            bind_addr,
            "kamn:did:agent:async-http-client-2",
            901,
            state_hash,
            "{\"message\":\"async-route-2\"}",
        );
        let (health, metrics, send_one, send_two) = tokio::join!(
            send_http_request_with_headers_async(bind_addr, "GET", "/healthz", "", &[]),
            send_http_request_with_headers_async(bind_addr, "GET", "/metrics", "", &[]),
            one,
            two,
        );
        AsyncTransportBurstResult {
            health,
            metrics,
            send_one,
            send_two,
        }
    })
}

async fn async_signed_send(
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    state_hash: &str,
    body: &str,
) -> Result<String, String> {
    let signature = service_api_request_signature_for_fields(sender_did, nonce, state_hash, body);
    let nonce_text = nonce.to_string();
    let headers = [
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", nonce_text.as_str()),
        ("X-KAMN-Request-Signature", signature.as_str()),
    ];
    send_http_request_with_headers_async(bind_addr, "POST", "/v1/messages/send", body, &headers)
        .await
}
