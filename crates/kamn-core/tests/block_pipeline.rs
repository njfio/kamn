use kamn_core::{
    bootstrap, BaselineTransaction, BlockConsensusRoundInput, BlockPipelineError,
    MempoolBlockPipeline, NodeConfig, NodeRole, SyncMode,
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

fn sample_tx(
    pipeline: &MempoolBlockPipeline,
    id: &str,
    sender: &str,
    nonce: u64,
) -> BaselineTransaction {
    BaselineTransaction::signed(
        id,
        sender,
        nonce,
        &format!("payload-{id}"),
        pipeline.expected_state_hash(),
    )
}

#[test]
fn functional_block_pipeline_commits_after_listener_and_approver_quorum() {
    let mut pipeline = MempoolBlockPipeline::new(true, 2, 2).expect("pipeline should initialize");
    pipeline
        .submit_transaction(sample_tx(&pipeline, "tx-1", "agent-a", 1))
        .expect("tx-1 should be accepted");
    pipeline
        .submit_transaction(sample_tx(&pipeline, "tx-2", "agent-b", 1))
        .expect("tx-2 should be accepted");

    let report = pipeline
        .run_consensus_round(BlockConsensusRoundInput {
            listener_event_id: "listener-event-1".to_owned(),
            listener_event_sequence: 1,
            outbound_action_id: "outbound-action-1".to_owned(),
            listener_votes: vec![
                (
                    "kamn:did:agent:listener-alpha".to_owned(),
                    "listen-att-1".to_owned(),
                ),
                (
                    "kamn:did:agent:listener-beta".to_owned(),
                    "listen-att-2".to_owned(),
                ),
            ],
            approver_votes: vec![
                (
                    "kamn:did:agent:approver-alpha".to_owned(),
                    "approve-att-1".to_owned(),
                    None,
                ),
                (
                    "kamn:did:agent:approver-beta".to_owned(),
                    "approve-att-2".to_owned(),
                    None,
                ),
            ],
        })
        .expect("consensus round should commit block");

    assert_eq!(report.block.height, 1);
    assert_eq!(report.block.transactions.len(), 2);
    assert!(report.listener_decision.accepted);
    assert!(report.approver_decision.authorized);
}

#[test]
fn integration_bootstrap_processor_wiring_includes_consensus_validator_component() {
    let plan = bootstrap(config_for(NodeRole::Processor, true)).expect("bootstrap should succeed");
    assert!(plan
        .wiring
        .all_components()
        .contains(&"consensus-validator"));
}

#[test]
fn regression_block_pipeline_rejects_payload_digest_mismatch_before_commit() {
    // Regression: #2927
    let mut pipeline = MempoolBlockPipeline::new(true, 1, 1).expect("pipeline should initialize");
    pipeline
        .submit_transaction(sample_tx(&pipeline, "tx-1", "agent-a", 1))
        .expect("transaction should be accepted");

    let result = pipeline.run_consensus_round(BlockConsensusRoundInput {
        listener_event_id: "listener-event-1".to_owned(),
        listener_event_sequence: 1,
        outbound_action_id: "outbound-action-1".to_owned(),
        listener_votes: vec![(
            "kamn:did:agent:listener-alpha".to_owned(),
            "listen-att-1".to_owned(),
        )],
        approver_votes: vec![(
            "kamn:did:agent:approver-alpha".to_owned(),
            "approve-att-1".to_owned(),
            Some("digest:mismatch".to_owned()),
        )],
    });

    assert!(matches!(
        result,
        Err(BlockPipelineError::ConsensusPayloadDigestMismatch { .. })
    ));
    assert_eq!(pipeline.processor_mempool_len(), 1);
}
