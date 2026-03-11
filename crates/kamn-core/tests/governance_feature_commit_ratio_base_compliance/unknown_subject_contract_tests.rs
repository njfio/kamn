use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_reports_unclassified_commit_subjects() {
    let (_output, report) = run_subject_checker(
        "6840-unknown-subject",
        &["miscellaneous update without prefix"],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "violation");
    assert_eq!(u64_field(&report, "unknown_commit_count"), 1);
    assert_eq!(
        string_field(&report, "reason_codes_csv"),
        "governance_commit_subject_unclassified"
    );
}
