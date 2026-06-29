use crate::support::constants::{REASON_CODES_CSV, REASON_TAXONOMY_VERSION, REPORT_SCHEMA_VERSION};
use crate::support::models::{Baseline, CurrentSurface, Evaluation, Thresholds};
use crate::support::paths::{fail, repo_path};
use std::fs;

pub(crate) fn maybe_write_report(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
    evaluation: &Evaluation,
) {
    let Ok(output_file) = std::env::var("KAMN_SHELL_TEST_SURFACE_RATIO_REPORT") else {
        return;
    };
    let output_path = repo_path(output_file.trim());
    let report = render_report(baseline, thresholds, current, evaluation);
    if let Err(error) = fs::write(&output_path, report) {
        let output_display = output_path.display();
        fail(
            "threshold_value_invalid",
            &format!("failed to write ratio report {output_display}: {error}"),
        );
    }
}

fn render_report(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
    evaluation: &Evaluation,
) -> String {
    let baseline_block = baseline_block(baseline);
    let current_block = current_block(current);
    let delta_block = delta_block(baseline, current);
    let thresholds_block = thresholds_block(thresholds);
    let policy_status = evaluation.policy_status;
    let final_decision = evaluation.final_decision;
    let reason_codes = report_reason_codes(evaluation);
    format!(
        "{{\n  \"schema_version\": \"{REPORT_SCHEMA_VERSION}\",\n  \"reason_taxonomy_version\": \"{REASON_TAXONOMY_VERSION}\",\n  \"reason_codes_csv\": \"{REASON_CODES_CSV}\",\n  \"policy_status\": \"{policy_status}\",\n  \"final_decision\": \"{final_decision}\",\n  \"reason_codes\": \"{reason_codes}\",\n  \"baseline\": {baseline_block},\n  \"current\": {current_block},\n  \"delta\": {delta_block},\n  \"thresholds\": {thresholds_block}\n}}\n"
    )
}

fn delta_fields(baseline: &Baseline, current: &CurrentSurface) -> (i64, i64, f64) {
    (
        current.shell_test_file_count - baseline.shell_test_file_count,
        current.rust_test_file_count - baseline.rust_test_file_count,
        current.shell_to_rust_ratio - baseline.shell_to_rust_ratio,
    )
}

fn waiver_path(thresholds: &Thresholds) -> String {
    thresholds
        .waiver_file
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "none".to_owned())
}

fn baseline_block(baseline: &Baseline) -> String {
    let shell_test_file_count = baseline.shell_test_file_count;
    let rust_test_file_count = baseline.rust_test_file_count;
    let docs_rust_test_file_count = baseline.docs_rust_test_file_count;
    let shell_to_rust_ratio = baseline.shell_to_rust_ratio;
    format!(
        "{{\n    \"shell_test_file_count\": {shell_test_file_count},\n    \"rust_test_file_count\": {rust_test_file_count},\n    \"docs_rust_test_file_count\": {docs_rust_test_file_count},\n    \"shell_to_rust_ratio\": {shell_to_rust_ratio:.6}\n  }}"
    )
}

fn current_block(current: &CurrentSurface) -> String {
    let shell_test_file_count = current.shell_test_file_count;
    let rust_test_file_count = current.rust_test_file_count;
    let docs_rust_test_file_count = current.docs_rust_test_file_count;
    let shell_to_rust_ratio = current.shell_to_rust_ratio;
    format!(
        "{{\n    \"shell_test_file_count\": {shell_test_file_count},\n    \"rust_test_file_count\": {rust_test_file_count},\n    \"docs_rust_test_file_count\": {docs_rust_test_file_count},\n    \"shell_to_rust_ratio\": {shell_to_rust_ratio:.6}\n  }}"
    )
}

fn delta_block(baseline: &Baseline, current: &CurrentSurface) -> String {
    let (shell_delta, rust_delta, ratio_delta) = delta_fields(baseline, current);
    format!(
        "{{\n    \"shell_test_file_delta\": {shell_delta},\n    \"rust_test_file_delta\": {rust_delta},\n    \"ratio_delta\": {ratio_delta:.6}\n  }}"
    )
}

fn thresholds_block(thresholds: &Thresholds) -> String {
    let allowed_shell_test_file_delta_max = thresholds.allowed_shell_test_file_delta_max;
    let allowed_ratio_delta_max = thresholds.allowed_ratio_delta_max;
    let waiver_file = waiver_path(thresholds);
    format!(
        "{{\n    \"allowed_shell_test_file_delta_max\": {allowed_shell_test_file_delta_max},\n    \"allowed_ratio_delta_max\": {allowed_ratio_delta_max:.6},\n    \"waiver_file\": \"{waiver_file}\"\n  }}"
    )
}

fn report_reason_codes(evaluation: &Evaluation) -> String {
    evaluation.reason_codes.join(",")
}
