use crate::support::repo_root;

const PR_BASE_ARGUMENT: &str = r#"--base-sha "${{ github.event.pull_request.base.sha }}""#;
const PR_HEAD_ARGUMENT: &str = r#"--head-sha "${{ github.event.pull_request.head.sha }}""#;
const PR_SUBJECT_RANGE: &str =
    r#""${{ github.event.pull_request.base.sha }}..${{ github.event.pull_request.head.sha }}""#;
const MORATORIUM_BASE_ARGUMENT: &str =
    r#"--base-sha "$GOVERNANCE_FEATURE_COMMIT_RATIO_MORATORIUM_BASE_SHA""#;

#[test]
fn workflow_evaluates_only_the_pull_request_commit_range() {
    let workflow = workflow_source();

    assert!(workflow.contains(PR_BASE_ARGUMENT));
    assert!(workflow.contains(PR_HEAD_ARGUMENT));
    assert!(workflow.contains(PR_SUBJECT_RANGE));
    assert!(!workflow.contains(MORATORIUM_BASE_ARGUMENT));
}

fn workflow_source() -> String {
    std::fs::read_to_string(repo_root().join(".github/workflows/ci-fast-gate.yml"))
        .expect("fast gate workflow should exist")
}
