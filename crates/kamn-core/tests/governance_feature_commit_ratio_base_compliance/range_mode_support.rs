use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::support::{read_report, temp_path};

pub struct TempGitRepo {
    root: PathBuf,
}

impl TempGitRepo {
    pub fn new(prefix: &str) -> Self {
        let root = temp_path(prefix, "repo");
        std::fs::create_dir_all(&root).expect("temp repo should be created");
        run_git(&root, &["init"]);
        run_git(&root, &["config", "user.name", "KAMN Test"]);
        run_git(
            &root,
            &["config", "user.email", "kamn-test@example.invalid"],
        );
        Self { root }
    }

    pub fn commit_file(&self, relative: &str, subject: &str) -> String {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("commit file parent should exist");
        }
        std::fs::write(&path, subject).expect("commit file should be written");
        run_git(&self.root, &["add", relative]);
        run_git(&self.root, &["commit", "-m", subject]);
        self.rev_parse("HEAD")
    }

    pub fn rev_parse(&self, rev: &str) -> String {
        run_git(&self.root, &["rev-parse", rev]).trim().to_owned()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

pub fn pr_range_with_commits(
    prefix: &str,
    governance_count: usize,
    feature_count: usize,
) -> (TempGitRepo, String, String) {
    let repo = TempGitRepo::new(prefix);
    let base = repo.commit_file("src/base.rs", "feat(7145): base");
    let mut head = base.clone();
    for index in 0..governance_count {
        let path = format!("specs/policy-{index}.md");
        head = repo.commit_file(&path, &format!("docs(7145): policy {index}"));
    }
    for index in 0..feature_count {
        let path = format!("src/feature-{index}.rs");
        head = repo.commit_file(&path, &format!("test(7145): feature {index}"));
    }
    (repo, base, head)
}

pub fn run_range_checker(
    repo_root: &Path,
    base_sha: &str,
    head_sha: &str,
    name: &str,
) -> (Output, Value) {
    let report_path = temp_path(name, "json");
    let output = Command::new("python3")
        .arg("scripts/ci/check_governance_feature_commit_ratio.py")
        .arg("--repo-root")
        .arg(repo_root)
        .arg("--base-sha")
        .arg(base_sha)
        .arg("--head-sha")
        .arg(head_sha)
        .arg("--window-size")
        .arg("50")
        .arg("--max-governance-ratio")
        .arg("0.20")
        .arg("--output-json")
        .arg(&report_path)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .expect("range checker should launch");
    (output, read_report(&report_path))
}

fn run_git(repo_root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .expect("git command should launch");
    assert!(
        output.status.success(),
        "git {:?} failed:
{}
{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git output should be utf8")
}
