use crate::support::{run_subject_checker, u64_field};

#[test]
fn checker_echoes_the_requested_window_size() {
    let (_output, report) = run_subject_checker(
        "6840-window-size-echo",
        &["feat(1): one"],
        "7",
        "0.20",
    );

    assert_eq!(u64_field(&report, "window_size"), 7);
}
