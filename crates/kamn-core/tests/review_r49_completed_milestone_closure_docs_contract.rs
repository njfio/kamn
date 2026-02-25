#[path = "review_doc_helpers.rs"]
mod review_doc_helpers;

use review_doc_helpers::{parse_marker_csv, parse_marker_usize};

const DOC: &str =
    include_str!("../../../docs/planning/2026-02-21-r49-completed-milestone-closure-wave.md");

#[test]
fn functional_r49_completed_milestone_closure_markers_present() {
    assert!(DOC.contains("completed_milestone_closure_wave_schema_version=kamn.review.completed-milestone-closure-wave.v1"));
    assert!(DOC.contains("completed_milestone_closure_wave_target_open_issue_count=0"));
    assert!(DOC.contains("completed_milestone_closure_wave_closed_milestone_ids_csv=94,95,96"));
    assert!(DOC.contains("completed_milestone_closure_wave_closed_milestone_count=3"));
    assert!(DOC.contains("completed_milestone_closure_wave_evidence_command_pre=gh api repos/njfio/kamn/milestones?state=open --paginate"));
    assert!(DOC.contains("completed_milestone_closure_wave_evidence_command_post=gh api repos/njfio/kamn/milestones?state=open --paginate"));
}

#[test]
fn integration_r49_completed_milestone_closure_markers_are_consistent() {
    let milestone_ids = parse_marker_csv(
        DOC,
        "completed_milestone_closure_wave_closed_milestone_ids_csv",
    );
    let milestone_count = parse_marker_usize(
        DOC,
        "completed_milestone_closure_wave_closed_milestone_count",
    );
    assert_eq!(milestone_ids.len(), milestone_count);
    assert_eq!(milestone_ids, vec!["94", "95", "96"]);
}
