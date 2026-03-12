use super::super::*;

#[test]
fn functional_rejoin_guard_accepts_matching_snapshot() {
    let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
    let attempt = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
    let status = guard.evaluate(attempt).expect("rejoin should be accepted");
    assert_eq!(status, RecoveryStatus::RejoinAccepted);
}

#[test]
fn integration_rejoin_guard_emits_catch_up_required_for_lagging_node() {
    let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
    let attempt = RejoinAttempt::new("node-a", 40, "state-40", "resume-1").expect("valid");
    let status = guard
        .evaluate(attempt)
        .expect("lagging node should receive catch-up guidance");
    assert_eq!(
        status,
        RecoveryStatus::CatchUpRequired {
            from_version: 40,
            to_version: 42
        }
    );
}

#[test]
fn unit_rejoin_guard_rejects_empty_resume_token() {
    let attempt = RejoinAttempt::new("node-a", 42, "state-42", "");
    assert_eq!(attempt, Err(RecoveryGuardError::InvalidResumeToken));
}

#[test]
fn regression_rejoin_replay_token_is_rejected() {
    let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
    let first = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
    assert_eq!(guard.evaluate(first), Ok(RecoveryStatus::RejoinAccepted));

    let replay = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
    let error = guard
        .evaluate(replay)
        .expect_err("replay token should be rejected");
    assert_eq!(
        error,
        RecoveryGuardError::ReplayResumeToken("resume-1".to_owned())
    );
}

#[test]
fn regression_rejoin_state_hash_mismatch_is_rejected() {
    let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
    let attempt = RejoinAttempt::new("node-a", 42, "state-41", "resume-1").expect("valid");
    let error = guard
        .evaluate(attempt)
        .expect_err("hash mismatch should be rejected");
    assert_eq!(
        error,
        RecoveryGuardError::StateHashMismatch {
            expected: "state-42".to_owned(),
            found: "state-41".to_owned()
        }
    );
}
