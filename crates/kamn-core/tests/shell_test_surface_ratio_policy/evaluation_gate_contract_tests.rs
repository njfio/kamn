use crate::support::constants::{REASON_CODES_CSV, REASON_TAXONOMY_VERSION};
use crate::support::paths::repo_path;
use crate::support::{
    current_surface, evaluate_policy, load_baseline, load_thresholds, maybe_write_report,
};

#[test]
fn functional_shell_test_surface_ratio_non_regression_gate() {
    let baseline = load_baseline(&repo_path(
        "fixtures/ci/shell_test_surface_ratio_baseline.env",
    ));
    let thresholds = load_thresholds(&repo_path(".ci/shell_test_surface_ratio_thresholds.env"));
    let current = current_surface();
    let evaluation = evaluate_policy(&baseline, &thresholds, &current);
    maybe_write_report(&baseline, &thresholds, &current, &evaluation);
    let reason_codes = evaluation.reason_codes.join(",");
    let shell_test_file_count = current.shell_test_file_count;
    let rust_test_file_count = current.rust_test_file_count;
    let shell_to_rust_ratio = current.shell_to_rust_ratio;
    let baseline_shell_test_file_count = baseline.shell_test_file_count;
    let baseline_rust_test_file_count = baseline.rust_test_file_count;
    let baseline_shell_to_rust_ratio = baseline.shell_to_rust_ratio;
    assert_ne!(
        evaluation.final_decision,
        "NO-GO",
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_codes={reason_codes} shell_test_file_count={shell_test_file_count} rust_test_file_count={rust_test_file_count} shell_to_rust_ratio={shell_to_rust_ratio:.6} baseline_shell_test_file_count={baseline_shell_test_file_count} baseline_rust_test_file_count={baseline_rust_test_file_count} baseline_shell_to_rust_ratio={baseline_shell_to_rust_ratio:.6}",
    );
}
