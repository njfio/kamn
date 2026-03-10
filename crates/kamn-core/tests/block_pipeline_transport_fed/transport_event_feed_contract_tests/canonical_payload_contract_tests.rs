use super::super::support::*;

#[test]
fn functional_transport_canonical_candidate_payload_round_trip_from_commit_report() {
    let feed = InMemoryTransportMempoolFeed::new(build_valid_chain_transactions());
    let store = InMemoryCanonicalCommitStore::default();
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, AcceptAllForkChoiceHook)
        .expect("transport-fed pipeline should build");
    let report = pipeline.run_transport_consensus_round(sample_consensus_input()).expect("transport-fed round should commit");

    let payload = encode_transport_commit_report_payload(&report).expect("commit report payload should encode");
    let decoded = decode_transport_canonical_candidate_payload(payload.as_str()).expect("canonical candidate payload should decode");
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

    let error = encode_transport_canonical_candidate_payload(&record).expect_err("comma in transaction id must fail closed");
    assert!(matches!(error, BlockPipelineError::TransportFeed(detail) if detail.contains("transport_candidate_transaction_id_invalid")), "invalid transaction id should carry deterministic reason-code marker");
}
