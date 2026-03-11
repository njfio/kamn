use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_counts_governance_only_subject_windows_correctly() {
    let (_output, report) = run_subject_checker(
        "6840-governance-only-subjects",
        &["docs(1): spec", "chore(1): cleanup", "spec(1): plan"],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "violation");
    assert_eq!(u64_field(&report, "governance_commit_count"), 3);
    assert_eq!(u64_field(&report, "feature_commit_count"), 0);
}
