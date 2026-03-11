use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_rejects_unknown_commit_prefixes() {
    let (_output, report) = run_subject_checker(
        "6840-unknown-commit-type",
        &["merge: synthetic merge-like subject"],
        "50",
        "0.20",
    );

    assert_eq!(u64_field(&report, "unknown_commit_count"), 1);
    assert_eq!(string_field(&report, "status"), "violation");
}
