use crate::support::{run_subject_checker, string_field};

#[test]
fn checker_emits_none_reason_codes_for_successful_subject_windows() {
    let (_output, report) = run_subject_checker(
        "6840-success-reason-codes",
        &["feat(1): code"],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(string_field(&report, "reason_codes_csv"), "none");
}
