use kamn_core::{
    decode_transport_canonical_candidate_payload, encode_transport_candidate_payload,
    encode_transport_canonical_candidate_payload, encode_transport_commit_report_payload,
    BaselineTransaction, BlockConsensusRoundInput, BlockPipelineError, CanonicalCommitRecord,
    DeterministicCompetingBranchForkChoiceHook, ForkChoiceDecision, ForkChoiceHook,
    InMemoryCanonicalCommitStore, InMemoryPeerLifecycleTransport, InMemoryTransportMempoolFeed,
    MempoolBlockPipeline, NodeRole, PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleTransport,
    TransportCanonicalCandidateFeed, TransportEventMempoolFeed, TransportFedBlockPipeline,
    TransportMempoolFeed,
};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
struct RejectAllForkChoiceHook;

impl ForkChoiceHook for RejectAllForkChoiceHook {
    fn evaluate_candidate(
        &mut self,
        _record: &kamn_core::CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError> {
        Ok(ForkChoiceDecision::Reject {
            reason_code: "fork_choice_rejected_for_test".to_owned(),
        })
    }
}

fn sample_consensus_input() -> BlockConsensusRoundInput {
    BlockConsensusRoundInput {
        listener_event_id: "event-1".to_owned(),
        listener_event_sequence: 1,
        outbound_action_id: "outbound-1".to_owned(),
        listener_votes: vec![("kamn:did:listener:alpha".to_owned(), "att-1".to_owned())],
        approver_votes: vec![(
            "kamn:did:agent:approver-alpha".to_owned(),
            "att-1".to_owned(),
            None,
        )],
    }
}

fn build_valid_chain_transactions() -> Vec<BaselineTransaction> {
    let mut planner = MempoolBlockPipeline::new(true, 1, 1).expect("planner pipeline should build");
    let state_hash_one = planner.expected_state_hash().to_owned();
    let tx_one = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-1", &state_hash_one);
    planner
        .submit_transaction(tx_one.clone())
        .expect("first tx should be accepted");
    let state_hash_two = planner.expected_state_hash().to_owned();
    let tx_two = BaselineTransaction::signed("tx-2", "agent-a", 2, "payload-2", &state_hash_two);
    vec![tx_two, tx_one]
}

#[test]
fn functional_transport_fed_pipeline_orders_candidates_and_persists_commit() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, kamn_core::AcceptAllForkChoiceHook)
            .expect("transport-fed pipeline should build");

    let report = pipeline
        .run_transport_consensus_round(sample_consensus_input())
        .expect("transport-fed round should commit");
    let committed_ids = report
        .block
        .transactions
        .iter()
        .map(|tx| tx.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(committed_ids, vec!["tx-1", "tx-2"]);

    let commit_records = pipeline
        .list_canonical_commits()
        .expect("canonical commit list should load");
    assert_eq!(commit_records.len(), 1);
    assert_eq!(commit_records[0].payload_digest, report.payload_digest);
    assert_eq!(commit_records[0].transaction_ids, vec!["tx-1", "tx-2"]);
}

#[test]
fn integration_transport_fed_pipeline_respects_fork_choice_rejects() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, RejectAllForkChoiceHook)
            .expect("transport-fed pipeline should build");

    let result = pipeline.run_transport_consensus_round(sample_consensus_input());
    assert_eq!(
        result,
        Err(BlockPipelineError::ForkChoiceRejected {
            reason_code: "fork_choice_rejected_for_test".to_owned(),
        })
    );
    assert!(pipeline
        .list_canonical_commits()
        .expect("canonical commit list should load")
        .is_empty());
}

#[test]
fn regression_transport_fed_pipeline_rejects_empty_transport_feed() {
    let feed = InMemoryTransportMempoolFeed::new(Vec::new());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, kamn_core::AcceptAllForkChoiceHook)
            .expect("transport-fed pipeline should build");

    let result = pipeline.run_transport_consensus_round(sample_consensus_input());
    assert_eq!(result, Err(BlockPipelineError::EmptyMempool));
}

#[test]
fn performance_transport_fed_pipeline_commit_path_stays_within_local_budget() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, kamn_core::AcceptAllForkChoiceHook)
            .expect("transport-fed pipeline should build");

    let started = Instant::now();
    let _ = pipeline
        .run_transport_consensus_round(sample_consensus_input())
        .expect("transport-fed round should commit");
    assert!(
        started.elapsed() <= Duration::from_secs(2),
        "transport-fed commit path should remain bounded"
    );
}

#[test]
fn regression_competing_branch_fork_choice_prefers_stable_head_independent_of_candidate_order() {
    let mut hook_a = DeterministicCompetingBranchForkChoiceHook::default();
    let mut hook_b = DeterministicCompetingBranchForkChoiceHook::default();

    let branch_a = CanonicalCommitRecord {
        block_height: 9,
        producer_role: kamn_core::NodeRole::Processor,
        payload_digest: "digest-b".to_owned(),
        transaction_ids: vec!["tx-b".to_owned()],
    };
    let branch_b = CanonicalCommitRecord {
        block_height: 9,
        producer_role: kamn_core::NodeRole::Processor,
        payload_digest: "digest-a".to_owned(),
        transaction_ids: vec!["tx-a".to_owned()],
    };

    hook_a
        .evaluate_candidate(&branch_a)
        .expect("first branch should evaluate");
    hook_a
        .evaluate_candidate(&branch_b)
        .expect("second branch should evaluate");

    hook_b
        .evaluate_candidate(&branch_b)
        .expect("first branch should evaluate");
    hook_b
        .evaluate_candidate(&branch_a)
        .expect("second branch should evaluate");

    let head_a = hook_a
        .canonical_head()
        .expect("head should be assigned after competing candidates");
    let head_b = hook_b
        .canonical_head()
        .expect("head should be assigned after competing candidates");

    assert_eq!(head_a.payload_digest, "digest-a");
    assert_eq!(head_b.payload_digest, "digest-a");
}

#[test]
fn regression_transport_fed_pipeline_rejects_stale_candidate_against_seeded_head() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let seeded_head = CanonicalCommitRecord {
        block_height: 50,
        producer_role: kamn_core::NodeRole::Processor,
        payload_digest: "head-50".to_owned(),
        transaction_ids: vec!["head-tx".to_owned()],
    };

    let hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(seeded_head);
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("transport-fed pipeline should build");

    let result = pipeline.run_transport_consensus_round(sample_consensus_input());
    assert_eq!(
        result,
        Err(BlockPipelineError::ForkChoiceRejected {
            reason_code: "fork_choice_stale_block_height".to_owned(),
        })
    );
    assert!(pipeline
        .list_canonical_commits()
        .expect("canonical commit list should load")
        .is_empty());
}

#[test]
fn functional_transport_event_feed_decodes_inbox_frames_into_transactions() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let topic = "kamn/messages/v1";
    let sender = "peer-sender";
    let recipient = "peer-recipient";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Approver, vec![topic.to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(recipient, NodeRole::Processor, vec![topic.to_owned()])
                .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    let tx =
        BaselineTransaction::signed("tx-live-1", "agent-live", 1, "payload-live", "state:seed");
    let payload =
        encode_transport_candidate_payload(&tx).expect("transport payload should encode correctly");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, &payload)
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed =
        TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
            .expect("feed should build");
    let drained = feed
        .drain_pending_transactions()
        .expect("feed should decode one transaction");
    assert_eq!(drained, vec![tx]);
}

#[test]
fn regression_transport_event_feed_rejects_malformed_payload() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let topic = "kamn/messages/v1";
    let sender = "peer-alpha";
    let recipient = "peer-beta";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Listener, vec![topic.to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(recipient, NodeRole::Processor, vec![topic.to_owned()])
                .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, "txwire:v1|missing|fields")
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed =
        TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
            .expect("feed should build");
    let result = feed.drain_pending_transactions();
    assert!(
        matches!(result, Err(BlockPipelineError::TransportFeed(detail))
            if detail.contains("p2p_ingress_payload_line_malformed")),
        "malformed payload should fail with deterministic marker"
    );
}

#[test]
fn regression_transport_event_feed_rejects_topic_mismatch() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let sender = "peer-tx";
    let recipient = "peer-runtime";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Listener, vec!["kamn.tx.v1".to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn.tx.v1".to_owned()],
            )
            .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    let tx = BaselineTransaction::signed(
        "tx-topic-1",
        "agent-topic",
        1,
        "payload-topic",
        "state:seed",
    );
    let payload =
        encode_transport_candidate_payload(&tx).expect("transport payload should encode correctly");
    transport
        .send(
            PeerGossipFrame::new("kamn.topic.unexpected", sender, recipient, &payload)
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed = TransportEventMempoolFeed::new(
        transport,
        recipient,
        Some(vec!["kamn/messages/v1".to_owned()]),
    )
    .expect("feed should build");
    let result = feed.drain_pending_transactions();
    assert!(
        matches!(result, Err(BlockPipelineError::TransportFeed(detail))
            if detail.contains("transport_candidate_topic_mismatch")),
        "topic mismatch should fail with deterministic marker"
    );
}

#[test]
fn functional_transport_canonical_candidate_payload_round_trip_from_commit_report() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, kamn_core::AcceptAllForkChoiceHook)
            .expect("transport-fed pipeline should build");
    let report = pipeline
        .run_transport_consensus_round(sample_consensus_input())
        .expect("transport-fed round should commit");

    let payload = encode_transport_commit_report_payload(&report)
        .expect("commit report payload should encode");
    let decoded = decode_transport_canonical_candidate_payload(payload.as_str())
        .expect("canonical candidate payload should decode");

    assert_eq!(decoded.block_height, report.block.height);
    assert_eq!(decoded.payload_digest, report.payload_digest);
    assert_eq!(decoded.transaction_ids, vec!["tx-1", "tx-2"]);
}

#[test]
fn regression_transport_canonical_candidate_payload_rejects_invalid_transaction_id() {
    let record = CanonicalCommitRecord {
        block_height: 9,
        producer_role: NodeRole::Processor,
        payload_digest: "digest-9".to_owned(),
        transaction_ids: vec!["tx-1".to_owned(), "tx,2".to_owned()],
    };

    let error = encode_transport_canonical_candidate_payload(&record)
        .expect_err("comma in transaction id must fail closed");
    assert!(
        matches!(error, BlockPipelineError::TransportFeed(detail)
            if detail.contains("transport_candidate_transaction_id_invalid")),
        "invalid transaction id should carry deterministic reason-code marker"
    );
}

#[test]
fn functional_transport_event_feed_drains_canonical_candidates_from_block_topic_frames() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let topic = "kamn/blocks/v1";
    let sender = "peer-block-sender";
    let recipient = "peer-block-recipient";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Approver, vec![topic.to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(recipient, NodeRole::Processor, vec![topic.to_owned()])
                .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    let record = CanonicalCommitRecord {
        block_height: 17,
        producer_role: NodeRole::Processor,
        payload_digest: "digest-17".to_owned(),
        transaction_ids: vec!["tx-17".to_owned()],
    };
    let payload = encode_transport_canonical_candidate_payload(&record)
        .expect("canonical candidate payload should encode");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, payload.as_str())
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed =
        TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
            .expect("feed should build");
    let candidates = feed
        .drain_canonical_candidates()
        .expect("candidate drain should decode block frame");
    assert_eq!(candidates, vec![record]);
}
