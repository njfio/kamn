#[test]
fn performance_message_lifecycle_snapshot_roundtrip_stays_within_ci_budget() {
    let store = performance_snapshot_store(256, "perf");
    let snapshot = store.export_snapshot();
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
    let store = performance_snapshot_store(6000, "deep");
    let snapshot = store.export_snapshot();
    let mut restored = MessageLifecycleStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should pass");
}

fn performance_snapshot_store(count: usize, tag: &str) -> MessageLifecycleStore {
    let mut store = MessageLifecycleStore::new();
    for index in 0..count {
        register_default_message(&mut store, &format!("urn:uuid:msg-snapshot-{tag}-{index}"));
    }
    store
}
