use crate::support::{run_subject_checker, string_field, u64_field};

#[test]
fn checker_counts_feature_only_subject_windows_correctly() {
    let (_output, report) = run_subject_checker(
        "6840-feature-only-subjects",
        &[
            "feat(1): code",
            "refactor(1): cleanup",
            "test(1): coverage",
            "integrate(1): wiring",
        ],
        "50",
        "0.20",
    );

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(u64_field(&report, "governance_commit_count"), 0);
    assert_eq!(u64_field(&report, "feature_commit_count"), 4);
}
