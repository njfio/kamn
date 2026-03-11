use super::*;
use std::time::Instant;

#[test]
fn performance_message_lifecycle_snapshot_roundtrip_stays_within_ci_budget() {
    let snapshot = populated_snapshot(256, "urn:uuid:msg-snapshot-perf");
    let mut restored = MessageLifecycleStore::new();
    let start = Instant::now();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 250,
        "message lifecycle snapshot roundtrip exceeded CI budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled message lifecycle deep lane"]
fn performance_message_lifecycle_snapshot_deep_lane_stress() {
    let snapshot = populated_snapshot(6000, "urn:uuid:msg-snapshot-deep");
    let mut restored = MessageLifecycleStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
}

fn populated_snapshot(count: usize, prefix: &str) -> MessageLifecycleSnapshot {
    let mut store = MessageLifecycleStore::new();
    for index in 0..count {
        store
            .register(
                &format!("{prefix}-{index}"),
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
    }
    store.export_snapshot()
}
