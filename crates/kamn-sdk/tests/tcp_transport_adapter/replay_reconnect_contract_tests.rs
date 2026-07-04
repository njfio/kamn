use super::support::*;

#[test]
fn functional_tcp_adapter_reconnect_preserves_nonce_replay_guard_state() {
    let addr = free_addr();
    let (listener_adapter, sender_adapter) = adapter_pair(addr.as_str());
    let first = build_envelope(
        did("sender-reconnect"),
        did("listener-reconnect"),
        1,
        "state:reconnect",
        "first-connect",
    );
    let second = build_envelope(
        did("sender-reconnect"),
        did("listener-reconnect"),
        2,
        "state:reconnect",
        "second-connect",
    );
    assert_received_envelope(listener_adapter.clone(), &sender_adapter, first.clone());
    assert_received_envelope(listener_adapter, &sender_adapter, second.clone());
}

#[test]
fn integration_tcp_adapter_replay_nonce_is_rejected_across_reconnect() {
    let (listener_adapter, sender_adapter, first, replayed) = replay_case();
    assert_received_envelope(listener_adapter.clone(), &sender_adapter, first);
    let replay_thread = listen_once_in_thread(listener_adapter);
    wait_for_listener();
    sender_adapter
        .send(&replayed)
        .unwrap_or_else(|error| panic!("replay send failed: {error}"));
    assert_listener_error(
        replay_thread,
        SdkError::Conflict("tcp handshake replay detected"),
    );
}

fn assert_received_envelope(
    listener_adapter: TcpTransportAdapter,
    sender_adapter: &TcpTransportAdapter,
    expected: TcpSignedEnvelope,
) {
    let listener_thread = listen_once_in_thread(listener_adapter);
    wait_for_listener();
    sender_adapter
        .send(&expected)
        .unwrap_or_else(|error| panic!("send failed: {error}"));
    let received =
        join_listener(listener_thread).unwrap_or_else(|error| panic!("listen failed: {error}"));
    assert_eq!(received.envelope, expected);
}

fn replay_case() -> (
    TcpTransportAdapter,
    TcpTransportAdapter,
    TcpSignedEnvelope,
    TcpSignedEnvelope,
) {
    let addr = free_addr();
    let (listener_adapter, sender_adapter) = adapter_pair(addr.as_str());
    let first = build_envelope(
        did("sender-replay"),
        did("listener-replay"),
        9,
        "state:replay",
        "nonce-9-initial",
    );
    let replayed = build_envelope(
        did("sender-replay"),
        did("listener-replay"),
        9,
        "state:replay",
        "nonce-9-replayed",
    );
    (listener_adapter, sender_adapter, first, replayed)
}
