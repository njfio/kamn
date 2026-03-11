use crate::support::{run_subject_checker, u64_field};

#[test]
fn checker_limits_subject_mode_to_newest_window_entries() {
    let (_output, report) = run_subject_checker(
        "6840-window-cap",
        &[
            "feat(1): newest",
            "feat(1): second",
            "feat(1): third",
            "docs(1): fourth",
            "docs(1): fifth",
        ],
        "3",
        "0.20",
    );

    assert_eq!(u64_field(&report, "input_non_merge_commit_total"), 5);
    assert_eq!(u64_field(&report, "non_merge_commit_total"), 3);
    assert_eq!(u64_field(&report, "feature_commit_count"), 3);
}
