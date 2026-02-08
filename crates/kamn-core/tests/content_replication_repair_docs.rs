const DOC: &str = include_str!("../../../docs/foundation/content-replication-repair.md");

#[test]
fn doc_contains_policy_and_repair_scope() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ContentReplicationPolicy"));
    assert!(DOC.contains("ContentReplicationManager"));
    assert!(DOC.contains("ContentRepairAction"));
}

#[test]
fn doc_contains_health_and_retry_rules() {
    assert!(DOC.contains("## Availability and Repair Rules"));
    assert!(DOC.contains("Healthy"));
    assert!(DOC.contains("Degraded"));
    assert!(DOC.contains("Unavailable"));
    assert!(DOC.contains("suppresses duplicate repair actions while a repair is pending"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test content_replication_repair"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_duplicate_repair_suppression_rule() {
    // Regression: #167
    assert!(DOC.contains("suppresses duplicate repair actions while a repair is pending"));
}
