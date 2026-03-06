use std::{fs, path::PathBuf};

const REQUIRED_OUTAGE_MARKERS: [&str; 5] = [
    "GitHub Actions Outage Recovery",
    "https://www.githubstatus.com/",
    "HTTP 500",
    "git commit --allow-empty",
    "current head SHA",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn read_repo_text(rel_path: &str) -> String {
    let path = repo_root().join(rel_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("expected {} to be readable: {}", path.display(), error))
}

fn assert_recovery_markers(doc: &str, rel_path: &str) {
    for marker in REQUIRED_OUTAGE_MARKERS {
        assert!(
            doc.contains(marker),
            "expected outage recovery marker `{marker}` in {rel_path}"
        );
    }
}

#[test]
fn spec_c01_agents_policy_contains_ci_outage_recovery_markers() {
    let agents = read_repo_text("AGENTS.md");
    assert_recovery_markers(&agents, "AGENTS.md");
}

#[test]
fn spec_c02_contributing_policy_contains_ci_outage_recovery_markers() {
    let contributing = read_repo_text(".github/CONTRIBUTING.md");
    assert_recovery_markers(&contributing, ".github/CONTRIBUTING.md");
}
