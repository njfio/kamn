use std::path::{Path, PathBuf};

const CHECKER_COPY: &str = r#"git show "origin/${{ github.base_ref }}:scripts/ci/check_governance_feature_commit_ratio.py""#;
const SUPPORT_COPY: &str = r#"git show "origin/${{ github.base_ref }}:scripts/ci/governance_feature_commit_ratio_support.py""#;
const GIT_HELPER_COPY: &str =
    r#"git show "origin/${{ github.base_ref }}:scripts/ci/governance_feature_commit_ratio_git.py""#;
const CHECKER_EXECUTION: &str = r#"python3 "$tmp_dir/check_governance_feature_commit_ratio.py""#;

#[test]
fn fast_gate_copies_governance_ratio_git_helper() {
    let workflow =
        std::fs::read_to_string(workflow_path()).expect("ci-fast-gate workflow should be readable");

    assert_workflow_contains(
        &workflow,
        CHECKER_COPY,
        "Fast Gate should copy the governance ratio checker from the base branch",
    );
    assert_workflow_contains(
        &workflow,
        SUPPORT_COPY,
        "Fast Gate should copy governance ratio support from the base branch",
    );
    assert_workflow_contains(
        &workflow,
        GIT_HELPER_COPY,
        "Fast Gate should copy the governance ratio git helper from the base branch",
    );
    assert_workflow_contains(
        &workflow,
        CHECKER_EXECUTION,
        "Fast Gate should execute the copied governance ratio checker from the temp dir",
    );
}

#[test]
fn workspace_premerge_fetches_full_history_for_governance_range_tests() {
    let workflow =
        std::fs::read_to_string(workflow_path()).expect("ci-fast-gate workflow should be readable");
    let job = workflow_job_block(&workflow, "workspace-premerge-gate:");

    assert_workflow_contains(
        job,
        "uses: actions/checkout@v4",
        "Workspace Pre-Merge should use actions checkout",
    );
    assert_workflow_contains(
        job,
        "fetch-depth: 0",
        "Workspace Pre-Merge must fetch full history for branch governance tests",
    );
}

fn assert_workflow_contains(workflow: &str, marker: &str, message: &str) {
    assert!(workflow.contains(marker), "{message}");
}

fn workflow_job_block<'a>(workflow: &'a str, job_name: &str) -> &'a str {
    workflow
        .split_once(job_name)
        .unwrap_or_else(|| panic!("{job_name} should exist in workflow"))
        .1
}

fn workflow_path() -> PathBuf {
    repo_root().join(".github/workflows/ci-fast-gate.yml")
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("kamn-core manifest should live under crates/kamn-core")
}
