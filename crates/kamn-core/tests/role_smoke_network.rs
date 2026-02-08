use kamn_core::{
    bootstrap, BaselineTransaction, NodeConfig, NodeRole, RoleSmokeNetwork, SmokeError,
    TransactionGuardError,
};

fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: gossip_enabled,
    }
}

fn sample_tx(
    network: &RoleSmokeNetwork,
    id: &str,
    sender: &str,
    nonce: u64,
) -> BaselineTransaction {
    BaselineTransaction::signed(
        id,
        sender,
        nonce,
        &format!("payload-{id}"),
        network.expected_state_hash(),
    )
}

#[test]
fn functional_roles_complete_smoke_roundtrip_with_gossip() {
    let mut network = RoleSmokeNetwork::new(true);
    network
        .submit_transaction(sample_tx(&network, "tx-1", "agent-a", 1))
        .expect("transaction submit should succeed");

    assert!(network.gossip_reached_all_roles("tx-1"));

    let block = network
        .produce_block()
        .expect("block production should succeed");
    assert_eq!(block.height, 1);
    assert_eq!(block.transactions.len(), 1);
    assert_eq!(network.processor.committed_len(), 1);
    assert_eq!(network.listener.committed_len(), 1);
    assert_eq!(network.approver.committed_len(), 1);
}

#[test]
fn integration_bootstrap_role_plans_match_smoke_network_expectations() {
    let processor_plan =
        bootstrap(config_for(NodeRole::Processor, true)).expect("processor bootstrap succeeds");
    let listener_plan =
        bootstrap(config_for(NodeRole::Listener, true)).expect("listener bootstrap succeeds");
    let approver_plan =
        bootstrap(config_for(NodeRole::Approver, true)).expect("approver bootstrap succeeds");

    assert!(processor_plan
        .wiring
        .all_components()
        .contains(&"block-producer"));
    assert!(listener_plan
        .wiring
        .all_components()
        .contains(&"external-listener"));
    assert!(approver_plan
        .wiring
        .all_components()
        .contains(&"quorum-approver"));

    let mut network = RoleSmokeNetwork::new(processor_plan.config.enable_gossip);
    network
        .submit_transaction(sample_tx(&network, "tx-1", "agent-a", 1))
        .expect("transaction submit should succeed");
    let block = network
        .produce_block()
        .expect("block production should succeed");
    assert_eq!(block.transactions[0].id, "tx-1");
}

#[test]
fn regression_gossip_disabled_prevents_cross_role_propagation() {
    // Regression: #18
    let mut network = RoleSmokeNetwork::new(false);
    network
        .submit_transaction(sample_tx(&network, "tx-1", "agent-a", 1))
        .expect("transaction submit should succeed");

    assert!(!network.gossip_reached_all_roles("tx-1"));

    let block = network
        .produce_block()
        .expect("block production should succeed");
    assert_eq!(block.transactions.len(), 1);
    assert_eq!(network.processor.committed_len(), 1);
    assert_eq!(network.listener.committed_len(), 0);
    assert_eq!(network.approver.committed_len(), 0);
}

#[test]
fn integration_rejects_invalid_nonce_at_submit_boundary() {
    let mut network = RoleSmokeNetwork::new(true);
    assert_eq!(
        network.submit_transaction(sample_tx(&network, "tx-1", "agent-a", 0)),
        Err(SmokeError::Guard(TransactionGuardError::InvalidNonce(0)))
    );
}
