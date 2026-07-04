use super::super::support::*;

#[test]
fn functional_transport_fed_pipeline_orders_candidates_and_persists_commit() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, AcceptAllForkChoiceHook)
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
            reason_code: "fork_choice_rejected_for_test".to_owned()
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
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, AcceptAllForkChoiceHook)
            .expect("transport-fed pipeline should build");

    let result = pipeline.run_transport_consensus_round(sample_consensus_input());
    assert_eq!(result, Err(BlockPipelineError::EmptyMempool));
}

#[test]
fn performance_transport_fed_pipeline_commit_path_stays_within_local_budget() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline =
        TransportFedBlockPipeline::new(true, 1, 1, feed, store, AcceptAllForkChoiceHook)
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
