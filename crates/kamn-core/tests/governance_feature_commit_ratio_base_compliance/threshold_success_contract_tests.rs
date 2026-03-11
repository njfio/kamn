use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_accepts_exact_threshold_boundary() {
    let (_output, report) = run_subject_checker(
        "6840-threshold-success",
        &[
            "docs(1): spec",
            "feat(1): code",
            "refactor(1): cleanup",
            "test(1): coverage",
            "integrate(1): wiring",
        ],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(u64_field(&report, "governance_commit_count"), 1);
    assert_eq!(u64_field(&report, "feature_commit_count"), 4);
    assert_eq!(string_field(&report, "reason_codes_csv"), "none");
}
