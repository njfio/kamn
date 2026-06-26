use crate::support::compute_report_with_policy;
use crate::support::constants::{REPORT_SCHEMA_VERSION, THRESHOLD_SCHEMA_VERSION};
use crate::support::maybe_write_report;

#[test]
fn public_api_surface_report_schema_is_deterministic() {
    let (report, thresholds, status, reason_codes, rendered) = compute_report_with_policy();
    maybe_write_report(&rendered);
    assert!(report
        .modules
        .windows(2)
        .all(|pair| pair[0].module <= pair[1].module));
    assert_eq!(
        report.public_items_delta,
        report.total_public_items as i64 - report.baseline_total_public_items as i64
    );
    assert!(rendered.contains(&format!("report_schema_version={}", REPORT_SCHEMA_VERSION)));
    assert!(rendered.contains(&format!(
        "policy_schema_version={}",
        THRESHOLD_SCHEMA_VERSION
    )));
    assert!(rendered.contains(&format!("policy_status={}", status.as_marker())));
    assert!(rendered.contains(&format!("reason_codes={}", reason_codes)));
    assert!(rendered.contains(&format!(
        "warn_total_delta_max={}",
        thresholds.warn_total_delta_max
    )));
    assert!(rendered.contains(&format!(
        "fail_total_delta_max={}",
        thresholds.fail_total_delta_max
    )));
}
