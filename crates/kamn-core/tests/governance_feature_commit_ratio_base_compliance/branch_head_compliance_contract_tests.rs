use crate::range_mode_support::{run_range_checker, TempGitRepo};
use crate::support::status;

#[test]
fn branch_head_satisfies_governance_ratio_gate() {
    let repo = TempGitRepo::new("7145-compliant-pr-head");
    let base = repo.commit_file("src/base.rs", "feat(7145): base");
    let _ = repo.commit_file("specs/7145-policy.md", "docs(7145): policy");
    let _ = repo.commit_file("src/feature-a.rs", "test(7145): feature a");
    let _ = repo.commit_file("src/feature-b.rs", "fix(7145): feature b");
    let _ = repo.commit_file("src/feature-c.rs", "refactor(7145): feature c");
    let head = repo.commit_file("src/feature-d.rs", "integrate(7145): feature d");
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
