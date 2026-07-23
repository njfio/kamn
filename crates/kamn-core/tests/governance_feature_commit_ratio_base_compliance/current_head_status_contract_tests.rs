use crate::range_mode_support::{run_range_checker, TempGitRepo};
use crate::support::{f64_field, status, u64_field, MAX_GOVERNANCE_RATIO};
use serde_json::Value;

#[test]
fn current_branch_head_restores_ratio_compliance() {
    let repo = TempGitRepo::new("7145-current-pr-range");
    let base = repo.commit_file("src/base.rs", "feat(7145): base");
    let _ = repo.commit_file("specs/7145-policy.md", "docs(7145): policy");
    let _ = repo.commit_file("src/feature-a.rs", "test(7145): feature a");
    let _ = repo.commit_file("src/feature-b.rs", "fix(7145): feature b");
    let _ = repo.commit_file("src/feature-c.rs", "refactor(7145): feature c");
    let head = repo.commit_file("src/feature-d.rs", "integrate(7145): feature d");
    let (output, report) =
        run_range_checker(repo.root(), &base, &head, "7145-current-pr-range-report");

    assert!(
        output.status.success(),
        "checker stdout:
{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(status(&report), "ok");
    assert_report_invariants(&report);
    assert_eq!(u64_field(&report, "non_merge_commit_total"), 5);
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
