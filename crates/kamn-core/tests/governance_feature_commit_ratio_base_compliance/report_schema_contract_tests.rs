use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn subject_mode_emits_expected_schema_markers() {
    let (_output, report) = run_subject_checker(
        "6840-report-schema",
        &["feat(1): add runtime path"],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "schema_version"), "kamn.ci.governance-feature-commit-ratio-report.v1");
    assert_eq!(
        string_field(&report, "reason_taxonomy_version"),
        "kamn.ci.governance-feature-commit-ratio-reason-taxonomy.v1"
    );
    assert_eq!(u64_field(&report, "window_size"), 50);
}
