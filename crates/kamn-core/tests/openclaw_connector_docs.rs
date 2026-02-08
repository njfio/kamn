const DOC: &str = include_str!("../../../docs/foundation/openclaw-connector-reference-workflow.md");

#[test]
fn doc_contains_connector_contract_and_workflow_steps() {
    assert!(DOC.contains("## Connector Contract"));
    assert!(DOC.contains("registerOpenClawAgent(modelFamily)"));
    assert!(DOC.contains("runReferenceWorkflow(request)"));
    assert!(DOC.contains("1. send canonical message"));
    assert!(DOC.contains("3. create + release escrow"));
}

#[test]
fn doc_contains_validation_rules_and_fast_lane_command() {
    assert!(DOC.contains("## Validation and Error Handling Rules"));
    assert!(DOC.contains("Workflow target must expose `openclaw` capability."));
    assert!(DOC.contains("npm --prefix packages/kamn-sdk test"));
}

#[test]
fn regression_requires_empty_prompt_rejection_rule() {
    // Regression: #190
    assert!(DOC.contains("Empty prompt is rejected."));
}
