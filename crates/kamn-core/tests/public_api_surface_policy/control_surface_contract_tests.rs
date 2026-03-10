use crate::support::compute_report_with_policy;
use crate::support::models::PolicyStatus;

#[test]
fn public_api_surface_policy_enforces_warn_fail_contract() {
    let (report, thresholds, status, _reason_codes, _rendered) = compute_report_with_policy();
    assert!(matches!(status, PolicyStatus::Within | PolicyStatus::Warn | PolicyStatus::ExceptionApplied));
    assert!(
        report.public_items_delta <= thresholds.fail_total_delta_max
            || matches!(status, PolicyStatus::ExceptionApplied),
        "delta={} fail_max={} status={}",
        report.public_items_delta,
        thresholds.fail_total_delta_max,
        status.as_marker()
    );
}
