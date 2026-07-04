use crate::support::{f64_field, run_subject_checker};

#[test]
fn checker_echoes_the_requested_ratio_ceiling() {
    let (_output, report) =
        run_subject_checker("6840-max-ratio-echo", &["feat(1): one"], "50", "0.33");

    assert_eq!(f64_field(&report, "max_governance_ratio"), 0.33);
}
