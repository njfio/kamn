use kamn_core::{
    BaselineTransaction, BlockPipelineError, CanonicalCommitRecord,
    GossipFrameTransportMempoolFeed, GossipIngressAdapter, PeerGossipFrame, TransportMempoolFeed,
};
use std::time::{Duration, Instant};

fn tx_payload(tx: &BaselineTransaction) -> String {
    format!(
        "id={}\nsender={}\nnonce={}\nstate_hash={}\npayload={}\nsignature={}",
        tx.id, tx.sender, tx.nonce, tx.state_hash, tx.payload, tx.signature
    )
}

#[test]
fn unit_gossip_ingress_decodes_transaction_payload_with_deterministic_fields() {
    let tx = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-1", "state:genesis");
    let frame = PeerGossipFrame::new("kamn/messages/v1", "peer-a", "peer-b", &tx_payload(&tx))
        .expect("frame should build");

    let decoded =
        GossipIngressAdapter::decode_frame(&frame).expect("transaction decode should pass");
    let decoded_tx = decoded
        .into_transaction()
        .expect("decoded frame should normalize into transaction payload");
    assert_eq!(decoded_tx, tx);
}

#[test]
fn functional_gossip_ingress_decodes_transaction_and_block_frames_in_single_batch() {
    let tx = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-1", "state:genesis");
    let tx_frame = PeerGossipFrame::new("messages", "peer-a", "peer-b", &tx_payload(&tx))
        .expect("tx frame should build");
    let block_frame = PeerGossipFrame::new(
        "kamn/blocks/v1",
        "peer-b",
        "peer-a",
        "block_height=9\nproducer_role=processor\npayload_digest=digest-9\ntransaction_ids=tx-1,tx-2",
    )
    .expect("block frame should build");

    let batch = GossipIngressAdapter::decode_frames(&[tx_frame, block_frame])
        .expect("batch decode should pass for tx+block");
    assert_eq!(batch.transactions, vec![tx]);
    assert_eq!(
        batch.canonical_candidates,
        vec![CanonicalCommitRecord {
            block_height: 9,
            producer_role: kamn_core::NodeRole::Processor,
            payload_digest: "digest-9".to_owned(),
            transaction_ids: vec!["tx-1".to_owned(), "tx-2".to_owned()],
        }]
    );
}

#[test]
fn integration_gossip_frame_transport_feed_drains_transactions_and_canonical_candidates() {
    let tx = BaselineTransaction::signed("tx-1", "agent-a", 1, "payload-1", "state:genesis");
    let tx_frame = PeerGossipFrame::new("kamn/messages/v1", "peer-a", "peer-b", &tx_payload(&tx))
        .expect("tx frame should build");
    let block_frame = PeerGossipFrame::new(
        "kamn/blocks/v1",
        "peer-b",
        "peer-a",
        "block_height=10\nproducer_role=processor\npayload_digest=digest-10\ntransaction_ids=tx-1",
    )
    .expect("block frame should build");

    let mut feed = GossipFrameTransportMempoolFeed::new(vec![tx_frame, block_frame]);
    let drained = feed
        .drain_pending_transactions()
        .expect("transport feed should decode tx frame");
    assert_eq!(drained, vec![tx]);

    let canonical = feed.drain_canonical_candidates();
    assert_eq!(
        canonical,
        vec![CanonicalCommitRecord {
            block_height: 10,
            producer_role: kamn_core::NodeRole::Processor,
            payload_digest: "digest-10".to_owned(),
            transaction_ids: vec!["tx-1".to_owned()],
        }]
    );
}

#[test]
fn regression_gossip_ingress_rejects_invalid_signature_with_reason_code() {
    // Regression: #3415
    let frame = PeerGossipFrame::new(
        "kamn/messages/v1",
        "peer-a",
        "peer-b",
        "id=tx-1\nsender=agent-a\nnonce=1\nstate_hash=state:genesis\npayload=payload-1\nsignature=sig:tampered",
    )
    .expect("frame should build");

    let error = GossipIngressAdapter::decode_frame(&frame)
        .expect_err("tampered signature must fail closed");
    assert_eq!(error.reason_code(), "p2p_ingress_tx_signature_invalid");
}

#[test]
fn regression_gossip_ingress_rejects_malformed_payload_line_with_reason_code() {
    // Regression: #3415
    let frame = PeerGossipFrame::new(
        "kamn/messages/v1",
        "peer-a",
        "peer-b",
        "id=tx-1\nsender=agent-a\nnonce\nstate_hash=state:genesis\npayload=payload-1\nsignature=sig:placeholder",
    )
    .expect("frame should build");

    let error = GossipIngressAdapter::decode_frame(&frame)
        .expect_err("malformed key/value line must fail closed");
    assert_eq!(error.reason_code(), "p2p_ingress_payload_line_malformed");
}

#[test]
fn performance_gossip_ingress_decoding_stays_within_local_budget() {
    let mut frames = Vec::new();
    for nonce in 1..=256_u64 {
        let tx = BaselineTransaction::signed(
            &format!("tx-{nonce}"),
            "agent-a",
            nonce,
            &format!("payload-{nonce}"),
            "state:genesis",
        );
        let frame = PeerGossipFrame::new("messages", "peer-a", "peer-b", &tx_payload(&tx))
            .expect("frame should build");
        frames.push(frame);
    }

    let started = Instant::now();
    let decoded =
        GossipIngressAdapter::decode_frames(&frames).expect("bulk decode should remain bounded");
    assert_eq!(decoded.transactions.len(), 256);
    assert!(
        started.elapsed() <= Duration::from_secs(2),
        "gossip ingress decode should remain within local contract budget"
    );
}

#[test]
fn integration_transport_feed_wraps_ingress_error_reason_code_in_pipeline_error() {
    let frame = PeerGossipFrame::new(
        "kamn/messages/v1",
        "peer-a",
        "peer-b",
        "id=\nsender=agent-a\nnonce=1\nstate_hash=state:genesis\npayload=payload-1\nsignature=sig:broken",
    )
    .expect("frame should build");
    let mut feed = GossipFrameTransportMempoolFeed::new(vec![frame]);

    let error = feed
        .drain_pending_transactions()
        .expect_err("invalid ingress frame must surface deterministic transport feed error");
    assert!(
        matches!(error, BlockPipelineError::TransportFeed(detail) if detail.contains("p2p_ingress_payload_missing_field")),
        "transport feed error should include deterministic reason-code marker"
    );
}
