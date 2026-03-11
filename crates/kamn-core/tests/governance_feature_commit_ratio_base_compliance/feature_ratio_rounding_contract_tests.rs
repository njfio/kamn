use crate::support::{f64_field, run_subject_checker};

#[test]
fn checker_rounds_feature_ratio_to_schema_precision() {
    let (_output, report) = run_subject_checker(
        "6840-feature-ratio-rounding",
        &["docs(1): spec", "feat(1): code", "test(1): coverage"],
        "50",
        "0.50",
    );

    assert_eq!(f64_field(&report, "feature_ratio"), 0.666667);
}
