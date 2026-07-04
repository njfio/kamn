use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_flags_threshold_violation_when_governance_exceeds_limit() {
    let (_output, report) = run_subject_checker(
        "6840-threshold-violation",
        &[
            "docs(1): spec",
            "feat(1): code",
            "docs(1): guide",
            "test(1): coverage",
        ],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "violation");
    assert_eq!(u64_field(&report, "governance_commit_count"), 2);
    assert_eq!(u64_field(&report, "feature_commit_count"), 2);
    assert_eq!(
        string_field(&report, "reason_codes_csv"),
        "governance_commit_ratio_threshold_exceeded"
    );
}
