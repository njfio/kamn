const DOC: &str = include_str!("../../../docs/planning/kamn-data-layer-prd.docx.md");

fn marker_line(marker: &str) -> Option<&str> {
    DOC.lines().find(|line| line.contains(marker))
}

fn marker_backtick_values(line: &str) -> Vec<&str> {
    line.split('`')
        .enumerate()
        .filter_map(|(index, value)| (index % 2 == 1).then_some(value))
        .collect()
}

fn extract_issue_refs(line: &str) -> Vec<&str> {
    line.split(|ch: char| ch == ',' || ch.is_ascii_whitespace() || ch == '`')
        .filter(|token| token.starts_with('#') && token.len() > 1)
        .collect()
}

#[test]
fn standalone_decision_markers_present() {
    assert!(DOC.contains("## **0. R43 Standalone Decision Contract**"));
    assert!(DOC.contains("`data_layer_standalone_reason_taxonomy_version`"));
    assert!(DOC.contains("kamn.data-layer.standalone-decision.reason-taxonomy.v1"));
    assert!(DOC.contains("`data_layer_m11_operator_readiness_standalone_status`"));
    assert!(DOC.contains("`standalone_by_design`"));
    assert!(DOC.contains("`data_layer_m11_operator_readiness_standalone_reason_code`"));
    assert!(DOC.contains("`data_layer_m11_operator_readiness_meta_assessment`"));
    assert!(DOC.contains("`data_layer_prd_conformance_standalone_status`"));
    assert!(DOC.contains("`data_layer_prd_conformance_standalone_reason_code`"));
    assert!(DOC.contains("`data_layer_prd_conformance_meta_assessment`"));
}

#[test]
fn typed_did_backlog_marker_references_follow_up_issue() {
    let line = marker_line("`typed_did_migration_backlog_issue_ids`")
        .expect("typed DID backlog issue marker line should exist");
    let expected_issue = std::env::var("KAMN_TYPED_DID_BACKLOG_EXPECTED_ISSUE")
        .unwrap_or_else(|_| "#5223".to_owned());

    assert!(
        line.contains("#"),
        "typed DID backlog marker must include at least one issue reference"
    );
    assert!(
        line.contains(expected_issue.as_str()),
        "typed DID backlog marker should reference expected actionable issue: {expected_issue}"
    );
}

#[test]
fn typed_did_backlog_scope_marker_is_present() {
    let line = marker_line("`typed_did_migration_backlog_scope`")
        .expect("typed DID backlog scope marker line should exist");
    assert!(
        line.contains("non_data_layer_string_did_callsites"),
        "typed DID backlog scope marker should stay explicit"
    );
}

#[test]
fn typed_did_inventory_markers_are_present_and_parseable() {
    let schema_line = marker_line("`typed_did_migration_inventory_schema_version`")
        .expect("typed DID inventory schema marker line should exist");
    assert!(
        schema_line.contains("kamn.typed-did-migration.inventory.v1"),
        "typed DID inventory schema marker must stay stable"
    );

    let module_count_line =
        marker_line("`typed_did_migration_inventory_non_data_layer_module_count`")
            .expect("typed DID inventory module-count marker line should exist");
    let module_count_values = marker_backtick_values(module_count_line);
    let module_count = module_count_values
        .get(1)
        .copied()
        .unwrap_or("")
        .parse::<usize>()
        .expect("typed DID inventory module-count marker should be parseable as usize");
    assert!(
        module_count > 0,
        "typed DID inventory module count should be positive"
    );

    let callsite_count_line =
        marker_line("`typed_did_migration_inventory_non_data_layer_did_string_callsite_count`")
            .expect("typed DID inventory callsite-count marker line should exist");
    let callsite_count_values = marker_backtick_values(callsite_count_line);
    let callsite_count = callsite_count_values
        .get(1)
        .copied()
        .unwrap_or("")
        .parse::<usize>()
        .expect("typed DID inventory callsite-count marker should be parseable as usize");
    assert!(
        callsite_count > 0,
        "typed DID inventory callsite count should be positive"
    );
}

#[test]
fn typed_did_wave_issue_markers_reference_follow_up_subtasks() {
    let backlog_line = marker_line("`typed_did_migration_backlog_issue_ids`")
        .expect("typed DID backlog issue marker line should exist");
    let wave_line = marker_line("`typed_did_migration_wave_issue_ids`")
        .expect("typed DID wave issue marker line should exist");

    let expected_wave_issues = ["#5228", "#5229", "#5230"];
    for issue in expected_wave_issues {
        assert!(
            backlog_line.contains(issue),
            "typed DID backlog marker should include {issue}"
        );
        assert!(
            wave_line.contains(issue),
            "typed DID wave marker should include {issue}"
        );
    }

    for issue in extract_issue_refs(wave_line) {
        assert!(
            issue.starts_with('#')
                && issue.len() > 1
                && issue.chars().skip(1).all(|ch| ch.is_ascii_digit()),
            "typed DID wave issue marker references must use #<id> format: {issue}"
        );
    }
}
