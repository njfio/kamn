use crate::support::{f64_field, run_subject_checker, string_field};

#[test]
fn checker_accepts_subject_windows_that_hit_the_ratio_boundary() {
    let (_output, report) = run_subject_checker(
        "6840-mixed-subject-ratio",
        &[
            "docs(1): spec",
            "feat(1): code",
            "feat(1): second",
            "test(1): coverage",
            "refactor(1): cleanup",
            "integrate(1): wiring",
            "feat(1): third",
            "feat(1): fourth",
            "test(1): second-coverage",
            "docs(1): guide",
        ],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(f64_field(&report, "governance_ratio"), 0.2);
}
