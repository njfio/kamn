const PR_TEMPLATE: &str = include_str!("../../../.github/pull_request_template.md");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn pr_template_declares_mitigation_issue_link_contract() {
    assert!(PR_TEMPLATE.contains("shell_surface_mitigation_issue: #<issue-id>|None"));
}

#[test]
fn pr_template_declares_regression_requires_linked_mitigation_issue() {
    assert!(PR_TEMPLATE.contains(
        "regressed_with_waiver requires shell_surface_mitigation_issue to link #<issue-id>"
    ));
}

#[test]
fn ci_strategy_docs_include_shell_surface_mitigation_link_contract() {
    assert!(CI_STRATEGY_DOC.contains("shell_surface_mitigation_issue"));
    assert!(CI_STRATEGY_DOC.contains("#<issue-id>|None"));
    assert!(CI_STRATEGY_DOC.contains(
        "regressed_with_waiver` requires `shell_surface_mitigation_issue` to link `#<issue-id>"
    ));
}
