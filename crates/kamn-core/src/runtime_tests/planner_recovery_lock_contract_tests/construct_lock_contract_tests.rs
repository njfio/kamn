use super::super::*;

#[test]
fn functional_construct_lock_allows_acquire_then_renew_flow() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let lease = lock
        .acquire_for("processor-a")
        .expect("initial lease acquisition should succeed");
    let renewed = lock
        .renew("processor-a", lease.fencing_token())
        .expect("lease renewal should succeed");
    assert!(renewed.fencing_token() > lease.fencing_token());
}

#[test]
fn unit_construct_lock_rejects_empty_owner_id() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let error = lock
        .acquire_for("")
        .expect_err("empty owner id must be rejected");
    assert_eq!(error, ConstructLockError::InvalidOwnerId);
}

#[test]
fn regression_split_brain_lock_acquisition_is_rejected() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    assert!(lock.acquire_for("processor-a").is_ok());
    let error = lock
        .acquire_for("processor-b")
        .expect_err("second owner acquisition must be rejected");
    assert_eq!(
        error,
        ConstructLockError::LeaseAlreadyHeld {
            owner: "processor-a".to_owned()
        }
    );
}

#[test]
fn regression_stale_lease_renewal_is_rejected() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let lease = lock
        .acquire_for("processor-a")
        .expect("initial lease acquisition should succeed");
    let error = lock
        .renew("processor-a", lease.fencing_token().saturating_sub(1))
        .expect_err("stale fencing token must be rejected");
    assert_eq!(
        error,
        ConstructLockError::StaleFencingToken {
            expected: lease.fencing_token(),
            found: lease.fencing_token().saturating_sub(1)
        }
    );
}

#[test]
fn functional_construct_lock_supports_transfer_then_release_flow() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let lease = lock
        .acquire_for("processor-a")
        .expect("initial lease acquisition should succeed");
    let transferred = lock
        .transfer("processor-a", "processor-b", lease.fencing_token())
        .expect("lease transfer should succeed");
    assert_eq!(transferred.owner_id(), "processor-b");
    assert!(lock.release("processor-b", transferred.fencing_token()).is_ok());
}

#[test]
fn unit_construct_lock_rejects_release_for_non_owner() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let lease = lock
        .acquire_for("processor-a")
        .expect("initial lease acquisition should succeed");
    let error = lock
        .release("processor-b", lease.fencing_token())
        .expect_err("non-owner release must be rejected");
    assert_eq!(
        error,
        ConstructLockError::LeaseOwnerMismatch {
            expected: "processor-a".to_owned(),
            found: "processor-b".to_owned()
        }
    );
}

#[test]
fn integration_daemon_tick_requires_matching_active_lease() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let lease = lock
        .acquire_for("processor-a")
        .expect("initial lease acquisition should succeed");
    assert!(execute_processor_daemon_tick(&lock, "processor-a", lease.fencing_token(), 0).is_ok());
}

#[test]
fn regression_unauthorized_transfer_is_rejected() {
    let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let lease = lock
        .acquire_for("processor-a")
        .expect("initial lease acquisition should succeed");
    let error = lock
        .transfer("processor-b", "processor-c", lease.fencing_token())
        .expect_err("unauthorized transfer must be rejected");
    assert_eq!(
        error,
        ConstructLockError::LeaseOwnerMismatch {
            expected: "processor-a".to_owned(),
            found: "processor-b".to_owned()
        }
    );
}

#[test]
fn regression_daemon_tick_without_lease_is_rejected() {
    let lock = ConstructLockGuard::new(5).expect("construct lock should build");
    let error = execute_processor_daemon_tick(&lock, "processor-a", 1, 0)
        .expect_err("daemon tick without lease must be rejected");
    assert_eq!(error, ConstructLockError::NoLeaseForExecution);
}
