use super::super::*;

#[test]
fn regression_replay_guard_capacity_eviction_bounds_memory_and_releases_oldest_nonce() {
    let start = Instant::now();
    let mut guard = ServiceApiReplayGuard::new(3, Duration::from_secs(300));

    assert!(guard.record_nonce_if_fresh("kamn:did:agent:alice", 1, start));
    assert!(guard.record_nonce_if_fresh("kamn:did:agent:alice", 2, start + Duration::from_secs(1)));
    assert!(guard.record_nonce_if_fresh("kamn:did:agent:alice", 3, start + Duration::from_secs(2)));
    assert_eq!(guard.tracked_entry_count(), 3);

    assert!(guard.record_nonce_if_fresh("kamn:did:agent:alice", 4, start + Duration::from_secs(3)));
    assert_eq!(guard.tracked_entry_count(), 3);
    assert!(!guard.record_nonce_if_fresh(
        "kamn:did:agent:alice",
        1,
        start + Duration::from_secs(4)
    ));
}

#[test]
fn regression_replay_guard_ttl_eviction_rejects_only_within_active_window() {
    let start = Instant::now();
    let mut guard = ServiceApiReplayGuard::new(8, Duration::from_secs(2));

    assert!(guard.record_nonce_if_fresh("kamn:did:agent:bob", 9, start));
    assert!(!guard.record_nonce_if_fresh("kamn:did:agent:bob", 9, start + Duration::from_secs(1)));
    assert!(!guard.record_nonce_if_fresh("kamn:did:agent:bob", 9, start + Duration::from_secs(3)));
}

#[test]
fn regression_issue_6196_nonce_contract_rejects_post_ttl_replay_nonce_values() {
    let start = Instant::now();
    let mut guard = ServiceApiReplayGuard::new(8, Duration::from_secs(2));

    assert!(guard.record_nonce_if_fresh("kamn:did:agent:ivy", 50, start));
    assert!(!guard.record_nonce_if_fresh("kamn:did:agent:ivy", 50, start + Duration::from_secs(3)));
    assert!(!guard.record_nonce_if_fresh("kamn:did:agent:ivy", 49, start + Duration::from_secs(4)));
    assert!(guard.record_nonce_if_fresh("kamn:did:agent:ivy", 51, start + Duration::from_secs(5)));
}

#[test]
fn regression_replay_guard_seeded_nonce_rejects_stale_values_after_restart() {
    let start = Instant::now();
    let mut guard = ServiceApiReplayGuard::new(8, Duration::from_secs(60));
    guard.seed_sender_nonce_high_watermark("kamn:did:agent:carol", 42);

    assert!(!guard.record_nonce_if_fresh("kamn:did:agent:carol", 42, start));
    assert!(!guard.record_nonce_if_fresh(
        "kamn:did:agent:carol",
        41,
        start + Duration::from_secs(1)
    ));
    assert!(guard.record_nonce_if_fresh(
        "kamn:did:agent:carol",
        43,
        start + Duration::from_secs(2)
    ));
}
