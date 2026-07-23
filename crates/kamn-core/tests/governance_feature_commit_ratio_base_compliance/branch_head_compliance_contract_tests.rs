use crate::range_mode_support::{pr_range_with_commits, run_range_checker};
use crate::support::{status, string_field};

#[test]
fn branch_head_satisfies_governance_ratio_gate() {
    let (repo, base, head) = pr_range_with_commits("7145-compliant-pr-head", 1, 4);
    let (output, report) =
        run_range_checker(repo.root(), &base, &head, "7145-compliant-pr-head-report");

    assert!(
        output.status.success(),
        "checker should pass for compliant PR history, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(status(&report), "ok", "report: {report:#?}");
}

#[test]
fn branch_head_rejects_governance_ratio_above_policy() {
    let (repo, base, head) = pr_range_with_commits("7145-violating-pr-head", 1, 3);
    let (output, report) =
        run_range_checker(repo.root(), &base, &head, "7145-violating-pr-head-report");

    assert!(!output.status.success());
    assert_eq!(status(&report), "violation");
    assert_eq!(
        string_field(&report, "reason_codes_csv"),
        "governance_commit_ratio_threshold_exceeded"
    );
}
