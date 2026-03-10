use super::support::*;

#[test]
fn unit_libp2p_native_adapter_rejects_invalid_listen_multiaddr() {
    let error = invalid_runtime_config_error(
        "peer-native-invalid",
        "/ip4/127.0.0.1/tcp/invalid-port",
        vec!["/ip4/127.0.0.1/tcp/9201".to_owned()],
    );
    assert_eq!(error.reason_code(), "p2p_transport_libp2p_runtime_config_invalid");
}

#[test]
fn unit_libp2p_native_adapter_rejects_invalid_bootstrap_multiaddr() {
    let error = invalid_runtime_config_error(
        "peer-native-invalid-bootstrap",
        "/ip4/127.0.0.1/tcp/9540",
        vec!["/ip4/127.0.0.1/tcp/9541/invalid-proto".to_owned()],
    );
    assert_eq!(error.reason_code(), "p2p_transport_libp2p_runtime_config_invalid");
}

fn invalid_runtime_config_error(
    peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
) -> P2pTransportError {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        peer_id,
        listen_address,
        bootstrap_peers,
        vec!["messages".to_owned()],
        3,
    )
    .expect("base config should build");
    Libp2pLivePeerLifecycleTransport::new(config, P2pSwarmHarnessMode::DryRun)
        .expect_err("invalid libp2p config must fail")
}
