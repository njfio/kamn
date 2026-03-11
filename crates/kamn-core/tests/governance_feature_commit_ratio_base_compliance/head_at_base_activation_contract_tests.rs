use crate::range_mode_support::{run_range_checker, TempGitRepo};
use crate::support::{string_field, u64_field};

#[test]
fn range_mode_reports_head_at_activation_base_when_head_equals_base() {
    let repo = TempGitRepo::new("6840-head-at-base");
    let base = repo.commit_file("crates/kamn-core/tests/base.txt", "feat(6840): seed base");
    let (_output, report) = run_range_checker(repo.root(), &base, &base, "6840-head-at-base-report");

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(string_field(&report, "activation_scope_status"), "head_at_activation_base");
    assert_eq!(u64_field(&report, "non_merge_commit_total"), 0);
}
