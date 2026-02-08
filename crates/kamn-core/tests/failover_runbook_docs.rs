const RUNBOOK: &str = include_str!("../../../docs/foundation/multi-az-failover-runbook.md");

#[test]
fn runbook_contains_topology_section() {
    assert!(RUNBOOK.contains("## Multi-AZ Topology"));
    assert!(RUNBOOK.contains("AZ-a"));
    assert!(RUNBOOK.contains("AZ-b"));
    assert!(RUNBOOK.contains("AZ-c"));
}

#[test]
fn runbook_contains_failover_steps() {
    assert!(RUNBOOK.contains("## Processor Failover Procedure"));
    assert!(RUNBOOK.contains("1. Detect processor failure"));
    assert!(RUNBOOK.contains("2. Validate listener and approver quorum"));
    assert!(RUNBOOK.contains("3. Promote standby processor"));
    assert!(RUNBOOK.contains("4. Verify chain continuity"));
}

#[test]
fn runbook_contains_verification_checklist() {
    assert!(RUNBOOK.contains("## Verification Checklist"));
    assert!(RUNBOOK.contains("State hash continuity confirmed"));
    assert!(RUNBOOK.contains("No duplicate block production"));
}

#[test]
fn regression_requires_topology_bundle_command_surface() {
    // Regression: #579
    assert!(RUNBOOK.contains("## Topology Bundle Command Surface"));
    assert!(RUNBOOK.contains("scripts/deploy/generate_bundle.sh"));
    assert!(RUNBOOK.contains("scripts/deploy/preflight_topology.sh --bundle-file"));
}
