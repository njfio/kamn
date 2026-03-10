use super::support::*;

#[test]
fn functional_tcp_adapter_relays_signed_envelope_between_two_processes() {
    let addr = free_addr();
    let (listener_adapter, sender_adapter) = adapter_pair(addr.as_str());
    let expected_envelope = build_envelope(
        did("sender-func"),
        did("listener-func"),
        11,
        "state:functional",
        "functional-envelope",
    );
    let listener_thread = listen_once_in_thread(listener_adapter);
    wait_for_listener();
    sender_adapter
        .send(&expected_envelope)
        .unwrap_or_else(|error| panic!("sender adapter failed to send envelope: {error}"));
    let received = join_listener(listener_thread)
        .unwrap_or_else(|error| panic!("listener adapter failed: {error}"));
    assert_eq!(received.envelope, expected_envelope);
    assert!(received.peer_addr.starts_with("127.0.0.1:"));
}

#[test]
fn integration_tcp_adapter_rejects_oversized_wire_payload() {
    let addr = free_addr();
    let (listener_adapter, sender_adapter) = limited_listener_sender_pair(addr.as_str(), 16);
    let large_body = "x".repeat(64);
    let envelope = build_envelope(
        did("sender-large"),
        did("listener-large"),
        1,
        "state:large",
        large_body.as_str(),
    );
    let listener_thread = listen_once_in_thread(listener_adapter);
    wait_for_listener();
    sender_adapter
        .send(&envelope)
        .unwrap_or_else(|error| panic!("sender adapter failed: {error}"));
    assert_eq!(
        join_listener(listener_thread),
        Err(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "exceeds max wire bytes",
        })
    );
}
