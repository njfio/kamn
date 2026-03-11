use crate::support::{output_json, read_report, run_checker, status};

#[test]
fn branch_head_satisfies_governance_ratio_gate() {
    let report_path = output_json("6840-branch-head");
    let output = run_checker("HEAD", &report_path);
    let report = read_report(&report_path);

    assert!(
        output.status.success(),
        "checker should pass at branch head, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status(&report), "ok", "report: {report:#?}");
}
