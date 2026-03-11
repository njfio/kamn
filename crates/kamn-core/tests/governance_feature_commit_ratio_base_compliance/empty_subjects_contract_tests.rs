use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_rejects_empty_subject_windows() {
    let (_output, report) = run_subject_checker("6840-empty-subjects", &[], "50", "0.20");

    assert_eq!(string_field(&report, "status"), "violation");
    assert_eq!(u64_field(&report, "non_merge_commit_total"), 0);
    assert_eq!(
        string_field(&report, "reason_codes_csv"),
        "governance_commit_subjects_empty"
    );
}
