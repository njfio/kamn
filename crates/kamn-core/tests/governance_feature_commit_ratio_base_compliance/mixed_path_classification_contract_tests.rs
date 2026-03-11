use crate::range_mode_support::{run_range_checker, TempGitRepo};
use crate::support::{string_field, u64_field};

#[test]
fn range_mode_counts_mixed_surface_commits_as_feature() {
    let repo = TempGitRepo::new("6840-mixed-path");
    let base = repo.commit_file("crates/kamn-core/tests/base.txt", "feat(6840): base");
    let head = repo.commit_file("specs/6840-temp.md", "docs(6840): mixed path");
    std::fs::write(repo.root().join("crates/kamn-core/tests/feature.txt"), "feature").expect("feature file");
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo.root())
        .args(["add", "specs/6840-temp.md", "crates/kamn-core/tests/feature.txt"])
        .output()
        .expect("git add");
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(repo.root())
        .args(["commit", "--amend", "--no-edit"])
        .output()
        .expect("git amend");
    let head = repo.rev_parse("HEAD");
    let (_output, report) = run_range_checker(repo.root(), &base, &head, "6840-mixed-path-report");

    assert_eq!(string_field(&report, "status"), "ok");
    assert_eq!(u64_field(&report, "governance_commit_count"), 0);
    assert_eq!(u64_field(&report, "feature_commit_count"), 1);
}
