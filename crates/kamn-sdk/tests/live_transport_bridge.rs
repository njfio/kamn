#[path = "support/live_transport_bridge.rs"]
mod support;

use kamn_sdk::{BridgeId, KamnAgent, Message, SdkError};
use std::thread;

use support::{
    bridge_requests, deterministic_u64_tag, did, ensure_live_test_env, live_client,
    malformed_bridge_requests, reserve_loopback_addr, run_contract_server, wait_for_server_ready,
};

#[test]
fn spec_c09_live_transport_bridge_routes_execute_network_contract() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_contract_server(server_addr, bridge_requests()));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let source_message_id = client
        .send(Message {
            from: did("sender-bridge-live"),
            to: did("recipient-bridge-live"),
            body: "bridge contract payload".to_owned(),
            channel: None,
        })
        .expect("send should succeed");

    let submitted = client
        .submit_bridge(&source_message_id, "testnet")
        .expect("submit_bridge should succeed");
    assert_eq!(
        submitted.bridge_id,
        BridgeId(deterministic_u64_tag("bridge-local-abc"))
    );
    assert_eq!(submitted.bridge_status, "submitted");
    assert_eq!(submitted.target_message_id, None);
    assert_eq!(submitted.forward_tx_hash, None);

    let queried_submitted = client
        .get_bridge_status(&submitted.bridge_id)
        .expect("submitted bridge should be queryable");
    assert_eq!(queried_submitted, submitted);

    let forwarded = client
        .forward_bridge(&submitted.bridge_id)
        .expect("forward_bridge should succeed");
    assert_eq!(forwarded.bridge_status, "forwarded");
    assert_eq!(
        forwarded.target_message_id,
        Some(kamn_sdk::MessageId(deterministic_u64_tag(
            "msg-bridge-target-bridge-local-abc"
        )))
    );
    assert_eq!(
        forwarded.forward_tx_hash,
        Some("sha256:bridge-forwarded-bridge-local-abc".to_owned())
    );

    let queried_forwarded = client
        .get_bridge_status(&submitted.bridge_id)
        .expect("forwarded bridge should be queryable");
    assert_eq!(queried_forwarded, forwarded);

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "bridge route server should satisfy request budget"
    );
}

#[test]
fn regression_live_transport_bridge_status_rejects_unknown_alias_before_network() {
    ensure_live_test_env();
    let mut client = live_client("127.0.0.1:65535");

    assert_eq!(
        client.get_bridge_status(&BridgeId(404)),
        Err(SdkError::NotFound {
            entity: "bridge",
            id: "404".to_owned(),
        })
    );
    assert_eq!(
        client.forward_bridge(&BridgeId(404)),
        Err(SdkError::NotFound {
            entity: "bridge",
            id: "404".to_owned(),
        })
    );
}

#[test]
fn regression_live_transport_bridge_status_rejects_malformed_service_payload() {
    ensure_live_test_env();
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server =
        thread::spawn(move || run_contract_server(server_addr, malformed_bridge_requests()));
    wait_for_server_ready();

    let mut client = live_client(bind_addr.as_str());
    let source_message_id = client
        .send(Message {
            from: did("sender-bridge-live"),
            to: did("recipient-bridge-live"),
            body: "bridge contract payload".to_owned(),
            channel: None,
        })
        .expect("send should succeed");
    let submitted = client
        .submit_bridge(&source_message_id, "testnet")
        .expect("submit_bridge should succeed");

    assert_eq!(
        client.get_bridge_status(&submitted.bridge_id),
        Err(SdkError::TransportFailure(
            "service response missing required field"
        ))
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "malformed bridge status should satisfy request budget"
    );
}
