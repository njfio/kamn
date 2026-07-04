use crate::range_mode_support::{run_range_checker, TempGitRepo};
use crate::support::{string_field, u64_field};

#[test]
fn range_mode_reports_head_precedes_activation_base_for_older_heads() {
    let repo = TempGitRepo::new("6840-head-precedes-base");
    let first = repo.commit_file("crates/kamn-core/tests/first.txt", "feat(6840): first");
    let base = repo.commit_file("crates/kamn-core/tests/base.txt", "feat(6840): base");
    let (_output, report) =
        run_range_checker(repo.root(), &base, &first, "6840-head-precedes-report");

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(
        string_field(&report, "activation_scope_status"),
        "head_precedes_activation_base"
    );
    assert_eq!(u64_field(&report, "non_merge_commit_total"), 0);
}
