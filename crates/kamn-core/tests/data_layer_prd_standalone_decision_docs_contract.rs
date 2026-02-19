const DOC: &str = include_str!("../../../docs/planning/kamn-data-layer-prd.docx.md");

fn marker_line(marker: &str) -> Option<&str> {
    DOC.lines().find(|line| line.contains(marker))
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
