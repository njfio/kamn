use kamn_core::{KeyRecoveryManager, RecoveryError, RecoveryState};

fn manager() -> KeyRecoveryManager {
    KeyRecoveryManager::new(
        "key_v1",
        vec!["approver_a".to_owned(), "approver_b".to_owned()],
        2,
    )
    .expect("manager should initialize")
}

#[test]
fn compromised_key_reuse_is_blocked_after_revocation() {
    let mut recovery = manager();
    recovery
        .declare_compromised("suspected leak")
        .expect("compromise declaration should succeed");
    recovery
        .emergency_revoke()
        .expect("emergency revoke should succeed");
    assert_eq!(recovery.state(), RecoveryState::Revoked);

    assert_eq!(
        recovery.verify_key_use("key_v1"),
        Err(RecoveryError::CompromisedKeyReuse("key_v1".to_owned()))
    );
}

#[test]
fn recovery_requires_authorized_approvers_and_quorum() {
    let mut recovery = manager();
    recovery
        .declare_compromised("leak")
        .expect("compromise declaration should succeed");
    recovery
        .emergency_revoke()
        .expect("emergency revoke should succeed");

    recovery
        .propose_recovery("key_v2", "approver_a", 77)
        .expect("authorized proposer should succeed");
    assert_eq!(
        recovery.finalize_recovery(),
        Err(RecoveryError::InsufficientApprovals {
            required: 2,
            actual: 1,
        })
    );
    assert_eq!(
        recovery.approve_recovery("intruder"),
        Err(RecoveryError::UnauthorizedApprover("intruder".to_owned()))
    );
    recovery
        .approve_recovery("approver_b")
        .expect("second approver should satisfy quorum");
    recovery
        .finalize_recovery()
        .expect("finalize should succeed at quorum");

    assert_eq!(recovery.state(), RecoveryState::Active);
    assert_eq!(recovery.current_key_id(), "key_v2");
}

#[test]
fn replay_nonce_is_rejected_for_recovery_proposals() {
    let mut recovery = manager();
    recovery
        .declare_compromised("leak")
        .expect("compromise declaration should succeed");
    recovery
        .emergency_revoke()
        .expect("emergency revoke should succeed");
    recovery
        .propose_recovery("key_v2", "approver_a", 88)
        .expect("first proposal should succeed");
    recovery
        .approve_recovery("approver_b")
        .expect("second approval should succeed");
    recovery
        .finalize_recovery()
        .expect("finalize should succeed");
    recovery
        .declare_compromised("second incident")
        .expect("second compromise should succeed");
    recovery
        .emergency_revoke()
        .expect("second revoke should succeed");

    assert_eq!(
        recovery.propose_recovery("key_v3", "approver_a", 88),
        Err(RecoveryError::ReplayNonce(88))
    );
}
