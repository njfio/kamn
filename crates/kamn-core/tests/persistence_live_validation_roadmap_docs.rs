const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn roadmap_tracks_kolme_live_runtime_durable_component_guard_status() {
    assert!(ROADMAP.contains("Task #3078"));
    assert!(ROADMAP.contains("Task #3082"));
    assert!(ROADMAP.contains("channel-snapshot-store:file-default"));
    assert!(ROADMAP.contains("message-lifecycle-snapshot-store:file-default"));
    assert!(ROADMAP.contains("runtime-snapshot-store:file-default"));
}
