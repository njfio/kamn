use crate::support::{run_subject_checker, string_field};

#[test]
fn subject_file_mode_reports_subject_window_activation_scope() {
    let (_output, report) = run_subject_checker(
        "6840-activation-scope-subject-window",
        &["feat(1): one"],
        "50",
        "0.20",
    );

    assert_eq!(
        string_field(&report, "activation_scope_status"),
        "subject_window_only"
    );
}
