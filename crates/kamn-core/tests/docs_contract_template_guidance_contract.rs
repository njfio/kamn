const SUBTASK_TEMPLATE: &str = include_str!("../../../.github/ISSUE_TEMPLATE/subtask.md");

#[test]
fn conformance_subtask_template_contains_docs_contract_migration_checklist_markers() {
    assert!(SUBTASK_TEMPLATE.contains(
        "## Docs-Contract Matrix Migration Checklist (Required when docs-contract suites are touched)"
    ));
    assert!(SUBTASK_TEMPLATE
        .contains("docs_contract_matrix_migration_checklist_status=required-when-applicable"));
    assert!(SUBTASK_TEMPLATE
        .contains("docs_contract_matrix_case_inventory_status=declared-or-not-applicable"));
    assert!(SUBTASK_TEMPLATE.contains(
        "docs_contract_matrix_legacy_suite_retirement_status=verified-or-not-applicable"
    ));
}
