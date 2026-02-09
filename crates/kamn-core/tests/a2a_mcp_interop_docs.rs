const INTEROP: &str = include_str!("../../../docs/foundation/a2a-mcp-interoperability.md");

#[test]
fn interop_spec_contains_message_type_mapping() {
    assert!(INTEROP.contains("## Message Type Mapping"));
    assert!(INTEROP.contains("| KAMN Envelope Header | A2A Concept | MCP Concept | Notes |"));
    assert!(
        INTEROP.contains("| Request | task.invoke | tool_call | Deterministic request dispatch. |")
    );
    assert!(INTEROP
        .contains("| Response | task.result | tool_result | Deterministic completion response. |"));
    assert!(
        INTEROP.contains("| Event | event.notify | notification | Non-blocking external signal. |")
    );
}

#[test]
fn interop_spec_contains_task_lifecycle_mapping() {
    assert!(INTEROP.contains("## Task Lifecycle Mapping"));
    assert!(INTEROP.contains("| KAMN Task State | A2A Task State | MCP-Oriented Interpretation |"));
    assert!(INTEROP.contains("| Submitted | pending | Awaiting execution slot. |"));
    assert!(INTEROP.contains("| InProgress | running | Active execution in progress. |"));
    assert!(INTEROP.contains("| Completed | succeeded | Terminal success state. |"));
    assert!(INTEROP.contains("| Failed | failed | Terminal failure state. |"));
}

#[test]
fn interop_spec_contains_sdk_examples_and_fallback_rules() {
    assert!(INTEROP.contains("## Deterministic SDK Examples"));
    assert!(INTEROP.contains("Rust SDK mapping example"));
    assert!(INTEROP.contains("Python SDK mapping example"));
    assert!(INTEROP.contains("TypeScript SDK mapping example"));
    assert!(INTEROP.contains("## Limitations and Fallback Behavior"));
    assert!(INTEROP.contains("Unknown external type maps to ManualReview."));
    assert!(INTEROP.contains("Lossy mapping paths must emit interoperability warning metadata."));
}

#[test]
fn regression_requires_ambiguous_mapping_manual_review_rule() {
    // Regression: #177
    assert!(INTEROP.contains("Ambiguous mapping decision: ManualReview."));
}

#[test]
fn interop_spec_contains_a2a_mcp_conformance_harness_commands() {
    assert!(INTEROP.contains("## A2A/MCP Conformance Harness Evidence Contract (Issue #893)"));
    assert!(INTEROP.contains("run_a2a_mcp_conformance_harness.py"));
    assert!(INTEROP.contains("check_a2a_mcp_conformance_policy.sh"));
    assert!(INTEROP.contains("run_a2a_mcp_conformance_contract_lane.sh"));
    assert!(INTEROP.contains("a2a_mcp_conformance_reason_codes:GO:v1"));
}

#[test]
fn regression_requires_a2a_mcp_conformance_schema_drift_fail_closed_policy() {
    // Regression: #893
    assert!(INTEROP.contains("Regression: #893"));
}
