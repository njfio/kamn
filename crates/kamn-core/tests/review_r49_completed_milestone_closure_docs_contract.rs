const DOC: &str =
    include_str!("../../../docs/planning/2026-02-21-r49-completed-milestone-closure-wave.md");

fn parse_marker_usize(marker_key: &str) -> usize {
    let needle = format!("{marker_key}=");
    let line = DOC
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    let value = line
        .split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim();
    value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be an unsigned integer: {value}"))
}

fn parse_marker_csv(marker_key: &str) -> Vec<String> {
    let needle = format!("{marker_key}=");
    let line = DOC
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    line.split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .split(',')
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

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
    let milestone_ids =
        parse_marker_csv("completed_milestone_closure_wave_closed_milestone_ids_csv");
    let milestone_count =
        parse_marker_usize("completed_milestone_closure_wave_closed_milestone_count");
    assert_eq!(milestone_ids.len(), milestone_count);
    assert_eq!(milestone_ids, vec!["94", "95", "96"]);
}
