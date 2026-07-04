use super::support::*;

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
        matches!(error, BlockPipelineError::ReplayDrift { reason_code, .. } if reason_code == "canonical_replay_payload_digest_mismatch"),
        "payload drift should emit deterministic reason-code marker"
    );
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
