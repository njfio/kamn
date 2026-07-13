use super::*;

pub(super) fn concurrent_release_requests(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    path: &str,
    body: &str,
) -> Vec<String> {
    let bind_addr = reserve_loopback_addr();
    with_api_server(snapshot, bind_addr.as_str(), 2, |addr| {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        thread::scope(|scope| {
            [183_u64, 183_u64]
                .into_iter()
                .map(|nonce| {
                    spawn_release(scope, barrier.clone(), snapshot, addr, path, body, nonce)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(join_release)
                .collect()
        })
    })
}

pub(super) fn ordered_authenticated_release_race(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    path: &str,
    body: &str,
    gate: &crate::service_api_endpoint::TestPostAuthGate,
) -> Vec<String> {
    let bind_addr = reserve_loopback_addr();
    with_api_server(snapshot, bind_addr.as_str(), 2, |addr| {
        thread::scope(|scope| {
            let first = scope.spawn(|| send_release(snapshot, addr, path, body, 193));
            gate.expect_arrivals(1);
            let second = scope.spawn(|| send_release(snapshot, addr, path, body, 194));
            gate.expect_arrivals(2);
            gate.release();
            vec![join_release(first), join_release(second)]
        })
    })
}

fn send_release(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    addr: &str,
    path: &str,
    body: &str,
    nonce: u64,
) -> String {
    signed_release_http(snapshot, addr, path, body, nonce)
}

fn spawn_release<'scope>(
    scope: &'scope thread::Scope<'scope, '_>,
    barrier: std::sync::Arc<std::sync::Barrier>,
    snapshot: &'scope crate::service_api_endpoint::ServiceApiSnapshot,
    addr: &'scope str,
    path: &'scope str,
    body: &'scope str,
    nonce: u64,
) -> thread::ScopedJoinHandle<'scope, String> {
    scope.spawn(move || {
        barrier.wait();
        signed_release_http(snapshot, addr, path, body, nonce)
    })
}

fn signed_release_http(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    addr: &str,
    path: &str,
    body: &str,
    nonce: u64,
) -> String {
    let nonce_text = nonce.to_string();
    let signature =
        service_api_request_signature_for_fields(ACTOR, nonce, state_hash(snapshot).as_str(), body);
    send_http_request_with_headers(
        addr,
        "POST",
        path,
        body,
        &[
            ("X-KAMN-Sender-DID", ACTOR),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Authz-Scope", "escrow:write"),
        ],
    )
}

fn join_release(handle: thread::ScopedJoinHandle<'_, String>) -> String {
    handle.join().expect("release request thread")
}
