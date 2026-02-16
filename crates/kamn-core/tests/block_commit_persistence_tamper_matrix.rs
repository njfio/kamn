use kamn_core::{
    build_canonical_replay_evidence_bundle, BlockPipelineError, CanonicalCommitRecord, NodeRole,
};
use std::time::{Duration, Instant};

fn sample_canonical_record(height: u64, digest: &str, tx_id: &str) -> CanonicalCommitRecord {
    CanonicalCommitRecord {
        block_height: height,
        producer_role: NodeRole::Processor,
        payload_digest: digest.to_owned(),
        transaction_ids: vec![tx_id.to_owned()],
    }
}

#[test]
fn unit_replay_rejects_persisted_payload_digest_mismatch_reason_code() {
    let pre_restart = vec![sample_canonical_record(11, "digest-11", "tx-11")];
    let post_restart = vec![sample_canonical_record(11, "digest-11-tampered", "tx-11")];

    let error = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect_err("payload digest mismatch must fail closed");
    assert!(matches!(
        error,
        BlockPipelineError::ReplayDrift { reason_code, .. }
            if reason_code == "canonical_replay_payload_digest_mismatch"
    ));
}

#[test]
fn functional_replay_rejects_persisted_checkpoint_missing_reason_code() {
    let pre_restart = vec![
        sample_canonical_record(20, "digest-20", "tx-20"),
        sample_canonical_record(21, "digest-21", "tx-21"),
    ];
    let post_restart = vec![sample_canonical_record(20, "digest-20", "tx-20")];

    let error = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect_err("missing checkpoint must fail closed");
    assert!(matches!(
        error,
        BlockPipelineError::ReplayDrift { reason_code, .. }
            if reason_code == "canonical_replay_checkpoint_missing"
    ));
}

#[test]
fn integration_replay_tamper_matrix_emits_deterministic_reason_codes() {
    let baseline = sample_canonical_record(31, "digest-31", "tx-31");
    let pre_restart = vec![baseline.clone()];
    let tamper_cases = vec![
        (
            vec![sample_canonical_record(99, "digest-31", "tx-31")],
            "canonical_replay_block_height_mismatch",
        ),
        (
            vec![sample_canonical_record(31, "digest-31-tampered", "tx-31")],
            "canonical_replay_payload_digest_mismatch",
        ),
        (
            vec![CanonicalCommitRecord {
                block_height: 31,
                producer_role: NodeRole::Approver,
                payload_digest: "digest-31".to_owned(),
                transaction_ids: vec!["tx-31".to_owned()],
            }],
            "canonical_replay_producer_role_mismatch",
        ),
        (
            vec![CanonicalCommitRecord {
                block_height: 31,
                producer_role: NodeRole::Processor,
                payload_digest: "digest-31".to_owned(),
                transaction_ids: vec!["tx-31-tampered".to_owned()],
            }],
            "canonical_replay_transaction_ids_mismatch",
        ),
    ];

    for (tampered_post_restart, expected_reason_code) in tamper_cases {
        let error = build_canonical_replay_evidence_bundle(&pre_restart, &tampered_post_restart)
            .expect_err("tampered persisted artifact must fail closed");
        assert!(matches!(
            error,
            BlockPipelineError::ReplayDrift { reason_code, .. }
                if reason_code == expected_reason_code
        ));
    }
}

#[test]
fn regression_replay_height_mismatch_reason_code_stable() {
    // Regression: #4321
    let pre_restart = vec![sample_canonical_record(41, "digest-41", "tx-41")];
    let post_restart = vec![sample_canonical_record(42, "digest-41", "tx-41")];

    let first = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect_err("height mismatch must fail closed");
    let second = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect_err("height mismatch must fail closed");

    assert!(matches!(
        first,
        BlockPipelineError::ReplayDrift { ref reason_code, .. }
            if reason_code == "canonical_replay_block_height_mismatch"
    ));
    assert_eq!(first, second);
}

#[test]
fn performance_replay_tamper_matrix_stays_within_local_budget() {
    let pre_restart = (1..=256_u64)
        .map(|height| {
            sample_canonical_record(
                height,
                format!("digest-{height}").as_str(),
                format!("tx-{height}").as_str(),
            )
        })
        .collect::<Vec<_>>();
    let post_restart = pre_restart.clone();

    let started = Instant::now();
    let evidence = build_canonical_replay_evidence_bundle(&pre_restart, &post_restart)
        .expect("large replay lineage should validate");
    assert_eq!(evidence.post_restart_commit_count, 256);
    assert!(
        started.elapsed() <= Duration::from_secs(2),
        "replay tamper matrix exceeded local budget"
    );
}
