use crate::support::{current_surface, evaluate_policy, load_baseline, load_thresholds, maybe_write_report};
use crate::support::constants::{REASON_CODES_CSV, REASON_TAXONOMY_VERSION};
use crate::support::paths::repo_path;

#[test]
fn functional_shell_test_surface_ratio_non_regression_gate() {
    let baseline = load_baseline(&repo_path("fixtures/ci/shell_test_surface_ratio_baseline.env"));
    let thresholds = load_thresholds(&repo_path(".ci/shell_test_surface_ratio_thresholds.env"));
    let current = current_surface();
    let evaluation = evaluate_policy(&baseline, &thresholds, &current);
    maybe_write_report(&baseline, &thresholds, &current, &evaluation);
    assert_ne!(
        evaluation.final_decision,
        "NO-GO",
        "reason_taxonomy_version={} reason_codes_csv={} reason_codes={} shell_test_file_count={} rust_test_file_count={} shell_to_rust_ratio={:.6} baseline_shell_test_file_count={} baseline_rust_test_file_count={} baseline_shell_to_rust_ratio={:.6}",
        REASON_TAXONOMY_VERSION,
        REASON_CODES_CSV,
        evaluation.reason_codes.join(","),
        current.shell_test_file_count,
        current.rust_test_file_count,
        current.shell_to_rust_ratio,
        baseline.shell_test_file_count,
        baseline.rust_test_file_count,
        baseline.shell_to_rust_ratio,
    );
}
