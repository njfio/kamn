use kamn_core::{
    durable_commit_checker_reason_taxonomy_version, enforce_durable_commit_checker_lane_boundary,
    project_durable_commit_checker_reason, BlockPipelineError, DurableCommitCheckerLaneMode,
    DurableCommitCheckerReasonClass,
};
use std::time::{Duration, Instant};

#[test]
fn unit_replay_drift_reason_projection_is_deterministic() {
    let error = BlockPipelineError::ReplayDrift {
        reason_code: "canonical_replay_payload_digest_mismatch".to_owned(),
        detail: "payload digest drift".to_owned(),
    };

    let projection = project_durable_commit_checker_reason(&error);

    assert_eq!(
        projection.reason_code(),
        "canonical_replay_payload_digest_mismatch"
    );
    assert_eq!(
        projection.reason_class(),
        DurableCommitCheckerReasonClass::ReplayDrift
    );
    assert_eq!(
        projection.source_marker(),
        "durable_commit_checker_reason_projection"
    );
    assert_eq!(
        projection.reason_taxonomy_version(),
        durable_commit_checker_reason_taxonomy_version()
    );
}

#[test]
fn functional_ci_smoke_lane_boundary_emits_low_cost_markers() {
    let boundary = enforce_durable_commit_checker_lane_boundary(
        DurableCommitCheckerLaneMode::CiSmoke,
        true,
        false,
    )
    .expect("ci smoke boundary should pass");

    assert_eq!(boundary.final_decision, "GO");
    assert_eq!(boundary.ci_smoke_local_heavy_boundary_status, "verified");
    assert_eq!(boundary.ci_smoke_lane_cost_profile, "low");
    assert_eq!(boundary.local_heavy_lane_execution_mode, "not-applicable");
    assert_eq!(
        boundary.enforcement_reason_code,
        "durable_commit_checker_ci_smoke_boundary_verified"
    );
}

#[test]
fn integration_checker_projection_and_lane_boundary_contracts_are_consistent() {
    let boundary = enforce_durable_commit_checker_lane_boundary(
        DurableCommitCheckerLaneMode::LocalHeavy,
        false,
        true,
    )
    .expect("local-heavy boundary should pass with explicit opt-in");

    assert_eq!(boundary.final_decision, "GO");
    assert_eq!(boundary.local_heavy_lane_execution_mode, "opt_in");

    let error = BlockPipelineError::CommitStore(
        "canonical commit sqlite schema mismatch: expected 1, found 2 (canonical_commit_store_sqlite_schema_mismatch)"
            .to_owned(),
    );
    let projection = project_durable_commit_checker_reason(&error);

    assert_eq!(
        projection.reason_code(),
        "canonical_commit_store_sqlite_schema_mismatch"
    );
    assert_eq!(
        projection.reason_class(),
        DurableCommitCheckerReasonClass::CommitStore
    );
}

#[test]
fn regression_local_heavy_opt_in_reason_code_stays_stable() {
    // Regression: #4322
    let error = enforce_durable_commit_checker_lane_boundary(
        DurableCommitCheckerLaneMode::LocalHeavy,
        false,
        false,
    )
    .expect_err("local-heavy boundary should fail without explicit opt-in");

    assert_eq!(
        error.reason_code(),
        "durable_commit_checker_local_heavy_opt_in_required"
    );

    let projection = project_durable_commit_checker_reason(&error);
    assert_eq!(
        projection.reason_class(),
        DurableCommitCheckerReasonClass::LaneBoundary
    );
}

#[test]
fn performance_reason_projection_and_boundary_loops_stay_within_local_budget() {
    let start = Instant::now();

    for index in 0..50_000_u32 {
        let error = match index % 3 {
            0 => BlockPipelineError::ReplayDrift {
                reason_code: "canonical_replay_checkpoint_missing".to_owned(),
                detail: "checkpoint missing".to_owned(),
            },
            1 => BlockPipelineError::CommitStore(
                "canonical commit sqlite schema mismatch: expected 1, found 2 (canonical_commit_store_sqlite_schema_mismatch)"
                    .to_owned(),
            ),
            _ => BlockPipelineError::TransportFeed("transport feed timeout".to_owned()),
        };

        let projection = project_durable_commit_checker_reason(&error);
        assert!(
            !projection.reason_code().trim().is_empty(),
            "projection reason code must remain explicit"
        );

        let mode = if index % 2 == 0 {
            DurableCommitCheckerLaneMode::CiSmoke
        } else {
            DurableCommitCheckerLaneMode::LocalHeavy
        };
        let ci_fast_gate = index % 2 == 0;
        let local_opt_in = index % 2 == 1;

        let boundary =
            enforce_durable_commit_checker_lane_boundary(mode, ci_fast_gate, local_opt_in)
                .expect("loop boundary input should remain valid for selected mode");
        assert_eq!(boundary.final_decision, "GO");
    }

    assert!(
        start.elapsed() <= Duration::from_secs(2),
        "durable commit checker projection/boundary loop exceeded local budget"
    );
}
