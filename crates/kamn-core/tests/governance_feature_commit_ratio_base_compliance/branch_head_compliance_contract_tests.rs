use crate::range_mode_support::{pr_range_with_commits, run_range_checker};
use crate::support::status;

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
