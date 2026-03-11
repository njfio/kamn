use crate::support::{run_subject_checker, u64_field};

#[test]
fn checker_parses_scoped_commit_types_as_feature_commits() {
    let (_output, report) = run_subject_checker(
        "6840-scoped-commit-types",
        &["refactor(6840): tighten repair helpers"],
        "50",
        "0.20",
    );

    assert_eq!(u64_field(&report, "feature_commit_count"), 1);
    assert_eq!(u64_field(&report, "governance_commit_count"), 0);
}
