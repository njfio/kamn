pub(crate) use kamn_core::{
    build_canonical_replay_evidence_bundle, decode_transport_canonical_candidate_payload,
    encode_transport_candidate_payload, encode_transport_canonical_candidate_payload,
    encode_transport_commit_report_payload, AcceptAllForkChoiceHook, BaselineTransaction,
    BlockConsensusRoundInput, BlockPipelineError, CanonicalCandidateDecision,
    CanonicalCandidateOutcome, CanonicalCommitRecord, CanonicalCommitStore,
    CanonicalReplayEvidenceBundle, DeterministicCompetingBranchForkChoiceHook,
    FileCanonicalCommitStore, ForkChoiceDecision, ForkChoiceHook, InMemoryCanonicalCommitStore,
    InMemoryPeerLifecycleTransport, InMemoryTransportMempoolFeed, MempoolBlockPipeline, NodeRole,
    PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleTransport, TransportCanonicalCandidateFeed,
    TransportEventMempoolFeed, TransportFedBlockPipeline, TransportMempoolFeed,
};
pub(crate) use std::fs;
pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::OnceLock;
pub(crate) use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TEST_SIGNER_PRIVATE_KEY_A_HEX: &str =
    "7f2dcf2ef6bcf53b1af2359954f04eb6d25688fd87cbf09f7f9db4c6522f4c6b";

pub(crate) fn ensure_default_signer_key_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        std::env::set_var("KAMN_SIGNER_PRIVATE_KEY_HEX", TEST_SIGNER_PRIVATE_KEY_A_HEX);
        std::env::set_var(
            "KAMN_SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_HEX",
            TEST_SIGNER_PRIVATE_KEY_A_HEX,
        );
    });
}

#[derive(Debug, Default)]
pub(crate) struct RejectAllForkChoiceHook;

impl ForkChoiceHook for RejectAllForkChoiceHook {
    fn evaluate_candidate(
        &mut self,
        _record: &CanonicalCommitRecord,
    ) -> Result<ForkChoiceDecision, BlockPipelineError> {
        Ok(ForkChoiceDecision::Reject {
            reason_code: "fork_choice_rejected_for_test".to_owned(),
        })
    }
}

pub(crate) fn sample_consensus_input() -> BlockConsensusRoundInput {
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

pub(crate) fn build_valid_chain_transactions() -> Vec<BaselineTransaction> {
    ensure_default_signer_key_env();
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

pub(crate) fn sample_canonical_record(
    height: u64,
    digest: &str,
    tx_id: &str,
) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height: height,
        producer_role: NodeRole::Processor,
        payload_digest: digest.to_owned(),
        transaction_ids: vec![tx_id.to_owned()],
    }
}

pub(crate) fn temp_canonical_commit_store_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-canonical-commit-{tag}-{nonce}.log"))
}

pub(crate) fn cleanup_store_path(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

pub(crate) fn open_file_store(path: &PathBuf) -> FileCanonicalCommitStore {
    FileCanonicalCommitStore::new(path.clone()).expect("store should build")
}

pub(crate) fn advertise_transport_topic(
    transport: &InMemoryPeerLifecycleTransport,
    sender: &str,
    recipient: &str,
    topic: &str,
) {
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
}

pub(crate) fn persist_record(
    path: &PathBuf,
    record: CanonicalCommitRecord,
) -> Vec<CanonicalCommitRecord> {
    let mut store = open_file_store(path);
    store
        .persist_canonical_commit(record)
        .expect("baseline record should persist");
    store
        .list_canonical_commits()
        .expect("pre-restart list should load")
}

pub(crate) fn send_canonical_candidate(
    transport: &InMemoryPeerLifecycleTransport,
    topic: &str,
    sender: &str,
    recipient: &str,
    record: &CanonicalCommitRecord,
) {
    let payload = encode_transport_canonical_candidate_payload(record)
        .expect("canonical candidate payload should encode");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, payload.as_str())
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");
}

pub(crate) fn restart_replay_pipeline(
    path: &PathBuf,
    transport: InMemoryPeerLifecycleTransport,
    recipient: &str,
    topic: &str,
    canonical_head: CanonicalCommitRecord,
) -> TransportFedBlockPipeline<
    TransportEventMempoolFeed<InMemoryPeerLifecycleTransport>,
    FileCanonicalCommitStore,
    DeterministicCompetingBranchForkChoiceHook,
> {
    let feed = TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
        .expect("feed should build");
    let store = open_file_store(path);
    let hook = DeterministicCompetingBranchForkChoiceHook::with_canonical_head(canonical_head);
    TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("transport-fed pipeline should build")
}
