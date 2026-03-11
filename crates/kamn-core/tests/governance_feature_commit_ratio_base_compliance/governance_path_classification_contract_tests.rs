use crate::range_mode_support::{run_range_checker, TempGitRepo};
use crate::support::{string_field, u64_field};

#[test]
fn range_mode_counts_specs_only_commits_as_governance() {
    let repo = TempGitRepo::new("6840-governance-path");
    let base = repo.commit_file("crates/kamn-core/tests/base.txt", "feat(6840): base");
    let head = repo.commit_file("specs/6840-temp.md", "docs(6840): governance path");
    let (_output, report) = run_range_checker(repo.root(), &base, &head, "6840-governance-path-report");

    assert_eq!(string_field(&report, "status"), "violation");
    assert_eq!(u64_field(&report, "governance_commit_count"), 1);
    assert_eq!(u64_field(&report, "feature_commit_count"), 0);
}
