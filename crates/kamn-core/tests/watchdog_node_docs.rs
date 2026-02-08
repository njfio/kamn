const DOC: &str = include_str!("../../../docs/foundation/watchdog-node-prototype.md");

#[test]
fn doc_contains_watchdog_scope_and_detection_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("WatchdogNode"));
    assert!(DOC.contains("## Detection Rules"));
    assert!(DOC.contains("Invalid block parent"));
    assert!(DOC.contains("Quorum anomaly"));
    assert!(DOC.contains("Censorship signal"));
}

#[test]
fn doc_contains_snapshot_and_validation_semantics() {
    assert!(DOC.contains("## Snapshot Semantics"));
    assert!(DOC.contains("WatchdogSnapshot"));
    assert!(DOC.contains("## Validation and Error Handling"));
    assert!(DOC.contains("Config rejects zero quorum threshold."));
}

#[test]
fn regression_requires_single_recipient_censorship_exclusion_rule() {
    // Regression: #204
    assert!(
        DOC.contains("single-recipient deliveries are excluded from censorship classification.")
    );
}
