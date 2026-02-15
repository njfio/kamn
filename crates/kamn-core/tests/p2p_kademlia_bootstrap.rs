use kamn_core::{
    build_p2p_swarm_deterministic_config, compose_kademlia_discovery_bootstrap, NodeConfig,
    NodeRole, P2pSwarmHarnessMode, P2pSwarmHarnessTask, P2pTransportError, SyncMode,
};

fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: gossip_enabled,
        sync_mode: SyncMode::Fast,
    }
}

#[test]
fn unit_kademlia_bootstrap_rejects_empty_seed_set() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9100",
        vec![],
        vec!["messages".to_owned()],
        2,
    )
    .expect("swarm config should build");

    let result = compose_kademlia_discovery_bootstrap(&config);
    assert_eq!(
        result,
        Err(P2pTransportError::MissingKademliaBootstrapSeeds)
    );
}

#[test]
fn unit_kademlia_bootstrap_rejects_invalid_seed_multiaddr() {
    let result = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9100",
        vec!["127.0.0.1:9101".to_owned()],
        vec!["messages".to_owned()],
        2,
    );

    assert_eq!(
        result,
        Err(P2pTransportError::InvalidSwarmBootstrapPeerAddress(
            "127.0.0.1:9101".to_owned()
        ))
    );
}

#[test]
fn functional_kademlia_bootstrap_plan_normalizes_seed_order() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9100",
        vec![
            "/ip4/127.0.0.1/tcp/9102/p2p/peer-c".to_owned(),
            "/ip4/127.0.0.1/tcp/9101/p2p/peer-b".to_owned(),
            "/ip4/127.0.0.1/tcp/9101/p2p/peer-b".to_owned(),
        ],
        vec!["messages".to_owned(), "blocks".to_owned()],
        3,
    )
    .expect("swarm config should build");

    let plan = compose_kademlia_discovery_bootstrap(&config)
        .expect("kademlia bootstrap plan should build");
    assert_eq!(plan.discovery_backend(), "kademlia");
    assert_eq!(
        plan.seed_peers(),
        vec![
            "/ip4/127.0.0.1/tcp/9101/p2p/peer-b".to_owned(),
            "/ip4/127.0.0.1/tcp/9102/p2p/peer-c".to_owned(),
        ]
    );
}

#[test]
fn integration_kademlia_bootstrap_plan_composes_with_swarm_harness_startup() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9100",
        vec![
            "/ip4/127.0.0.1/tcp/9101/p2p/peer-listener".to_owned(),
            "/ip4/127.0.0.1/tcp/9102/p2p/peer-approver".to_owned(),
        ],
        vec!["messages".to_owned(), "blocks".to_owned()],
        3,
    )
    .expect("swarm config should build");

    let plan = compose_kademlia_discovery_bootstrap(&config)
        .expect("kademlia bootstrap plan should build");
    assert_eq!(plan.seed_peers().len(), 2);

    let task = P2pSwarmHarnessTask::new(config);
    let report = task
        .start(P2pSwarmHarnessMode::Run)
        .expect("swarm harness start should pass");
    assert!(report.started());
    assert_eq!(report.executed_ticks(), 3);
}

#[test]
fn regression_kademlia_bootstrap_requires_seeded_discovery_plan() {
    // Regression: #3319
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9100",
        vec![],
        vec!["messages".to_owned()],
        1,
    )
    .expect("swarm config should build");

    assert_eq!(
        compose_kademlia_discovery_bootstrap(&config),
        Err(P2pTransportError::MissingKademliaBootstrapSeeds)
    );
}
