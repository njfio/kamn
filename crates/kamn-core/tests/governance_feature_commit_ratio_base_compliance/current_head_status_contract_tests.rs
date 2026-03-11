use crate::support::{f64_field, output_json, read_report, run_checker, status, u64_field};

#[test]
fn current_branch_head_restores_ratio_compliance() {
    let report_path = output_json("6840-current-head");
    let output = run_checker("HEAD", &report_path);
    let report = read_report(&report_path);

    assert!(output.status.success(), "checker stdout:
{}", String::from_utf8_lossy(&output.stdout));
    assert_eq!(status(&report), "ok");
    assert!(f64_field(&report, "governance_ratio") <= 0.20);
    assert_eq!(u64_field(&report, "non_merge_commit_total"), 50);
}
