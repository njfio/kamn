use super::support::*;
#[test]
fn performance_channel_snapshot_roundtrip_stays_within_ci_budget() {
    let mut store = ChannelStore::new();
    for index in 0..256 {
        store
            .create_group(
                &format!("channel:group:perf-{index}"),
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    format!("kamn:did:agent:member-{index}"),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
    }

    let snapshot = store.export_snapshot();
    let mut restored = ChannelStore::new();
    let start = Instant::now();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < 300,
        "channel snapshot roundtrip exceeded CI budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled channel snapshot deep lane"]
fn performance_channel_snapshot_deep_lane_stress() {
    let mut store = ChannelStore::new();
    for index in 0..6000 {
        store
            .create_group(
                &format!("channel:group:deep-{index}"),
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    format!("kamn:did:agent:member-{index}"),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
    }

    let snapshot = store.export_snapshot();
    let mut restored = ChannelStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
}
