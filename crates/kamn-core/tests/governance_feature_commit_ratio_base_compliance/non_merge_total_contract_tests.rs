use crate::support::{run_subject_checker, u64_field};

#[test]
fn checker_reports_windowed_non_merge_commit_total() {
    let (_output, report) = run_subject_checker(
        "6840-non-merge-total",
        &["feat(1): one", "feat(1): two", "feat(1): three", "feat(1): four"],
        "2",
        "0.20",
    );

    assert_eq!(u64_field(&report, "non_merge_commit_total"), 2);
}
