use crate::support::{
    f64_field, output_json, read_report, run_checker, status, u64_field, MAX_GOVERNANCE_RATIO,
    WINDOW_SIZE,
};
use serde_json::Value;

#[test]
fn current_branch_head_restores_ratio_compliance() {
    let report_path = output_json("6840-current-head");
    let output = run_checker("HEAD", &report_path);
    let report = read_report(&report_path);

    assert!(
        output.status.success(),
        "checker stdout:
{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(status(&report), "ok");
    assert_report_invariants(&report);
    let window = WINDOW_SIZE.parse::<u64>().expect("valid window size");
    assert_eq!(u64_field(&report, "non_merge_commit_total"), window);
}

fn assert_report_invariants(report: &Value) {
    let total = u64_field(report, "non_merge_commit_total");
    let governance = u64_field(report, "governance_commit_count");
    let feature = u64_field(report, "feature_commit_count");
    let unknown = u64_field(report, "unknown_commit_count");
    assert_eq!(governance + feature + unknown, total);
    assert_ratio(report, "governance_ratio", governance, total);
    assert_ratio(report, "feature_ratio", feature, total);
    let maximum = MAX_GOVERNANCE_RATIO.parse::<f64>().expect("valid maximum");
    assert!(f64_field(report, "governance_ratio") <= maximum);
}

fn assert_ratio(report: &Value, field: &str, count: u64, total: u64) {
    let expected = count as f64 / total as f64;
    let actual = f64_field(report, field);
    assert!((actual - expected).abs() < 1e-12, "{field}: {actual}");
}
