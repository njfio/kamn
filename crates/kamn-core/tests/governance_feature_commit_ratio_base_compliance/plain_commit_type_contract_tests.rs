use crate::support::{run_subject_checker, u64_field};

#[test]
fn checker_parses_plain_commit_types_without_scope() {
    let (_output, report) = run_subject_checker(
        "6840-plain-commit-types",
        &["feat: route feature without scope"],
        "50",
        "0.20",
    );

    assert_eq!(u64_field(&report, "feature_commit_count"), 1);
    assert_eq!(u64_field(&report, "governance_commit_count"), 0);
}
