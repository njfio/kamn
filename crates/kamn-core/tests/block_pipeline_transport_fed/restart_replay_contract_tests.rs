use super::support::*;

#[test]
fn functional_transport_fed_restart_replay_harness_reports_boundary_and_checkpoint_markers() {
    let path = temp_canonical_commit_store_path("restart-replay-functional");
    cleanup_store_path(&path);

    let mut store = open_file_store(&path);
    store
        .persist_canonical_commit(sample_canonical_record(11, "digest-11", "tx-11"))
        .expect("first record should persist");
    store
        .persist_canonical_commit(sample_canonical_record(12, "digest-12", "tx-12"))
        .expect("second record should persist");
    let pre_restart = store
        .list_canonical_commits()
        .expect("pre-restart list should load");

    let restarted_store = open_file_store(&path);
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

    cleanup_store_path(&path);
}

#[test]
fn integration_transport_fed_restart_replay_preserves_canonical_lineage_across_restart() {
    let path = temp_canonical_commit_store_path("restart-replay-integration");
    cleanup_store_path(&path);
    let (pre_restart, mut pipeline) = build_restart_replay_pipeline(&path);
    assert_duplicate_reconciliation(
        pipeline
            .reconcile_transport_candidates()
            .expect("reconciliation should succeed"),
    );
    let post_restart = pipeline
        .list_canonical_commits()
        .expect("post-restart list should load");
    assert_replay_lineage(pre_restart, post_restart);
    cleanup_store_path(&path);
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
            matches!(error, BlockPipelineError::ReplayDrift { reason_code, .. } if reason_code == expected_reason_code),
            "unexpected reason code for tamper case: {case_name}"
        );
    }
}

fn build_restart_replay_pipeline(
    path: &Path,
) -> (
    Vec<CanonicalCommitRecord>,
    TransportFedBlockPipeline<
        TransportEventMempoolFeed<InMemoryPeerLifecycleTransport>,
        FileCanonicalCommitStore,
        DeterministicCompetingBranchForkChoiceHook,
    >,
) {
    let baseline_record = sample_canonical_record(21, "digest-21", "tx-21");
    let pre_restart = persist_record(path, baseline_record.clone());
    let (transport, recipient, topic) = seeded_restart_transport(&baseline_record);
    let pipeline =
        build_duplicate_replay_pipeline(path, pre_restart.as_slice(), transport, recipient, topic);
    (pre_restart, pipeline)
}

fn seeded_restart_transport(
    baseline_record: &CanonicalCommitRecord,
) -> (InMemoryPeerLifecycleTransport, &'static str, &'static str) {
    let topic = "kamn/blocks/v1";
    let sender = "peer-replay-sender";
    let recipient = "peer-replay-recipient";
    let transport = InMemoryPeerLifecycleTransport::default();
    advertise_transport_topic(&transport, sender, recipient, topic);
    send_canonical_candidate(&transport, topic, sender, recipient, baseline_record);
    (transport, recipient, topic)
}

fn build_duplicate_replay_pipeline(
    path: &Path,
    pre_restart: &[CanonicalCommitRecord],
    transport: InMemoryPeerLifecycleTransport,
    recipient: &str,
    topic: &str,
) -> TransportFedBlockPipeline<
    TransportEventMempoolFeed<InMemoryPeerLifecycleTransport>,
    FileCanonicalCommitStore,
    DeterministicCompetingBranchForkChoiceHook,
> {
    restart_replay_pipeline(
        path,
        transport,
        recipient,
        topic,
        pre_restart
            .last()
            .cloned()
            .expect("baseline canonical head should exist"),
    )
}

fn assert_duplicate_reconciliation(outcomes: Vec<CanonicalCandidateOutcome>) {
    assert_eq!(outcomes.len(), 1);
    assert_eq!(
        outcomes[0].decision,
        CanonicalCandidateDecision::Rejected {
            reason_code: "fork_choice_duplicate_candidate".to_owned(),
        }
    );
}

fn assert_replay_lineage(
    pre_restart: Vec<CanonicalCommitRecord>,
    post_restart: Vec<CanonicalCommitRecord>,
) {
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("replay evidence should validate after duplicate reconciliation");
    assert_eq!(evidence.continuity_status, "verified");
    assert_eq!(evidence.post_restart_commit_count, 1);
}
