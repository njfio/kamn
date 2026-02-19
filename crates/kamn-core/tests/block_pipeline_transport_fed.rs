use kamn_core::{
    build_canonical_replay_evidence_bundle, decode_transport_canonical_candidate_payload,
    encode_transport_candidate_payload, encode_transport_canonical_candidate_payload,
    encode_transport_commit_report_payload, BaselineTransaction, BlockConsensusRoundInput,
    BlockPipelineError, CanonicalCommitRecord, CanonicalCommitStore, CanonicalReplayEvidenceBundle,
    DeterministicCompetingBranchForkChoiceHook, FileCanonicalCommitStore, ForkChoiceDecision,
    ForkChoiceHook, InMemoryCanonicalCommitStore, InMemoryPeerLifecycleTransport,
    InMemoryTransportMempoolFeed, MempoolBlockPipeline, NodeRole, PeerDiscoveryRecord,
    PeerGossipFrame, PeerLifecycleTransport, TransportCanonicalCandidateFeed,
    TransportEventMempoolFeed, TransportFedBlockPipeline, TransportMempoolFeed,
};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

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
        listener_votes: vec![(
            "kamn:did:agent:listener-alpha".to_owned(),
            "att-1".to_owned(),
        )],
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

fn sample_canonical_record(height: u64, digest: &str, tx_id: &str) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height: height,
        producer_role: NodeRole::Processor,
        payload_digest: digest.to_owned(),
        transaction_ids: vec![tx_id.to_owned()],
    }
}

fn temp_canonical_commit_store_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-canonical-commit-{tag}-{nonce}.log"))
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

#[test]
fn unit_canonical_replay_checkpoint_validator_accepts_matching_lineage() {
    let pre_restart = vec![
        sample_canonical_record(7, "digest-7", "tx-7"),
        sample_canonical_record(8, "digest-8", "tx-8"),
    ];
    let post_restart = pre_restart.clone();
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("matching lineage should validate");

    assert_eq!(
        evidence.schema_version,
        "kamn.runtime.canonical-replay-evidence.v1"
    );
    assert_eq!(evidence.restart_boundary_block_height, 8);
    assert_eq!(evidence.replay_checkpoint_block_height, 8);
    assert_eq!(evidence.continuity_status, "verified");
}

#[test]
fn unit_canonical_replay_checkpoint_validator_rejects_payload_digest_drift_reason_code() {
    let pre_restart = vec![sample_canonical_record(9, "digest-9", "tx-9")];
    let post_restart = vec![sample_canonical_record(9, "digest-9-tampered", "tx-9")];

    let error = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect_err("payload drift must fail closed");
    assert!(
        matches!(error, BlockPipelineError::ReplayDrift { reason_code, .. }
            if reason_code == "canonical_replay_payload_digest_mismatch"),
        "payload drift should emit deterministic reason-code marker"
    );
}

#[test]
fn functional_transport_fed_restart_replay_harness_reports_boundary_and_checkpoint_markers() {
    let path = temp_canonical_commit_store_path("restart-replay-functional");
    let _ = fs::remove_file(&path);

    let mut store = FileCanonicalCommitStore::new(path.clone()).expect("store should build");
    store
        .persist_canonical_commit(sample_canonical_record(11, "digest-11", "tx-11"))
        .expect("first record should persist");
    store
        .persist_canonical_commit(sample_canonical_record(12, "digest-12", "tx-12"))
        .expect("second record should persist");
    let pre_restart = store
        .list_canonical_commits()
        .expect("pre-restart list should load");

    let restarted_store = FileCanonicalCommitStore::new(path.clone()).expect("store should build");
    let post_restart = restarted_store
        .list_canonical_commits()
        .expect("post-restart list should load");
    let evidence: CanonicalReplayEvidenceBundle =
        build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
            .expect("restart replay evidence should validate");
    assert_eq!(evidence.restart_boundary_block_height, 12);
    assert_eq!(evidence.replay_checkpoint_block_height, 12);
    assert_eq!(evidence.pre_restart_commit_count, 2);
    assert_eq!(evidence.post_restart_commit_count, 2);

    let _ = fs::remove_file(path);
}

#[test]
fn integration_transport_fed_restart_replay_preserves_canonical_lineage_across_restart() {
    let path = temp_canonical_commit_store_path("restart-replay-integration");
    let _ = fs::remove_file(&path);
    let topic = "kamn/blocks/v1";
    let sender = "peer-replay-sender";
    let recipient = "peer-replay-recipient";
    let baseline_record = sample_canonical_record(21, "digest-21", "tx-21");

    let mut first_store = FileCanonicalCommitStore::new(path.clone()).expect("store should build");
    first_store
        .persist_canonical_commit(baseline_record.clone())
        .expect("baseline record should persist");
    let pre_restart = first_store
        .list_canonical_commits()
        .expect("pre-restart list should load");

    let transport = InMemoryPeerLifecycleTransport::default();
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

    let payload = encode_transport_canonical_candidate_payload(&baseline_record)
        .expect("canonical candidate payload should encode");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, payload.as_str())
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let feed = TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
        .expect("feed should build");
    let restarted_store = FileCanonicalCommitStore::new(path.clone()).expect("store should build");
    let hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(
        pre_restart
            .last()
            .cloned()
            .expect("baseline canonical head should exist"),
    );
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, restarted_store, hook)
        .expect("transport-fed pipeline should build");

    let outcomes = pipeline
        .reconcile_transport_candidates()
        .expect("reconciliation should succeed");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(
        outcomes[0].decision,
        kamn_core::CanonicalCandidateDecision::Rejected {
            reason_code: "fork_choice_duplicate_candidate".to_owned(),
        }
    );

    let post_restart = pipeline
        .list_canonical_commits()
        .expect("post-restart list should load");
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("replay evidence should validate after duplicate reconciliation");
    assert_eq!(evidence.continuity_status, "verified");
    assert_eq!(evidence.post_restart_commit_count, 1);

    let _ = fs::remove_file(path);
}

#[test]
fn regression_transport_fed_restart_replay_tamper_matrix_emits_deterministic_reason_codes() {
    let baseline = sample_canonical_record(31, "digest-31", "tx-31");
    let pre_restart = vec![baseline.clone()];
    let tamper_cases = vec![
        (
            vec![],
            "canonical_replay_checkpoint_missing".to_owned(),
            "missing checkpoint",
        ),
        (
            vec![sample_canonical_record(99, "digest-31", "tx-31")],
            "canonical_replay_block_height_mismatch".to_owned(),
            "height drift",
        ),
        (
            vec![sample_canonical_record(31, "digest-31-tampered", "tx-31")],
            "canonical_replay_payload_digest_mismatch".to_owned(),
            "payload digest drift",
        ),
        (
            vec![CanonicalCommitRecord {
                transaction_ids: vec!["tx-31-tampered".to_owned()],
                ..baseline.clone()
            }],
            "canonical_replay_transaction_ids_mismatch".to_owned(),
            "transaction id drift",
        ),
    ];

    for (tampered_post_restart, expected_reason_code, case_name) in tamper_cases {
        let error = build_canonical_replay_evidence_bundle(&pre_restart, &tampered_post_restart)
            .expect_err("tampered replay lineage must fail closed");
        assert!(
            matches!(error, BlockPipelineError::ReplayDrift { reason_code, .. }
                if reason_code == expected_reason_code),
            "unexpected reason code for tamper case: {case_name}"
        );
    }
}

#[test]
fn performance_canonical_replay_checkpoint_validator_stays_within_local_budget() {
    let mut pre_restart = Vec::new();
    let mut post_restart = Vec::new();
    for index in 1..=256 {
        pre_restart.push(sample_canonical_record(
            index,
            format!("digest-{index}").as_str(),
            format!("tx-{index}").as_str(),
        ));
        post_restart.push(sample_canonical_record(
            index,
            format!("digest-{index}").as_str(),
            format!("tx-{index}").as_str(),
        ));
    }

    let start = Instant::now();
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("large replay lineage should validate");
    assert_eq!(evidence.pre_restart_commit_count, 256);
    assert!(
        start.elapsed() <= Duration::from_secs(1),
        "canonical replay validator exceeded runtime budget"
    );
}
