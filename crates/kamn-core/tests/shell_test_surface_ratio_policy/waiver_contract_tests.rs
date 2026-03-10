use crate::support::constants::WAIVER_SCHEMA_VERSION;
use crate::support::load_waiver;
use crate::support::paths::repo_root;
use std::fs;

#[test]
fn regression_waiver_mitigation_issue_marker_must_match_issue_format() {
    let repo_tmp = repo_root().join("target/tmp/shell-test-surface-ratio");
    let _ = fs::remove_dir_all(&repo_tmp);
    fs::create_dir_all(&repo_tmp).expect("failed to create tmp ratio fixture directory");
    let invalid_waiver = repo_tmp.join("invalid-waiver.env");
    fs::write(
        &invalid_waiver,
        format!(
            "schema_version={}\nmitigation_issue=not-an-issue-id\nmax_shell_test_file_delta=10\nmax_ratio_delta=0.05\n",
            WAIVER_SCHEMA_VERSION
        ),
    )
    .expect("failed to write invalid waiver fixture");

    let panic_result = std::panic::catch_unwind(|| {
        let _ = load_waiver(&invalid_waiver);
    });
    assert!(panic_result.is_err(), "invalid waiver mitigation issue format must fail closed");
}
