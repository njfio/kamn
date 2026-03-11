use crate::support::{run_subject_checker, string_field};

#[test]
fn checker_emits_threshold_reason_code_for_ratio_failures() {
    let (_output, report) = run_subject_checker(
        "6840-violation-reason-codes",
        &["docs(1): spec", "docs(1): guide", "feat(1): code"],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "violation");
    assert_eq!(
        string_field(&report, "reason_codes_csv"),
        "governance_commit_ratio_threshold_exceeded"
    );
}
