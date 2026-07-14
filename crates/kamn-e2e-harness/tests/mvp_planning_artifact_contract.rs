use std::path::PathBuf;

const BRAINSTORM: &str = "docs/brainstorms/2026-06-26-kamn-forward-strategy-requirements.md";
const MVP_PLAN: &str = "docs/plans/2026-06-26-001-kamn-mvp-demo-readiness-plan.md";
const RAIL_PLAN: &str = "docs/plans/2026-07-10-001-kamn-agent-transaction-rail-mega-plan.md";
const ARTIFACTS: [(&str, &str); 3] = [
    (BRAINSTORM, "superseded"),
    (MVP_PLAN, "superseded"),
    (RAIL_PLAN, "completed"),
];

#[test]
fn spec_c01_historical_artifacts_are_explicitly_bounded() {
    for (path, status) in ARTIFACTS {
        require_markers(path, &["artifact_status: historical"]);
        require_markers(path, &[&format!("current_status: {status}")]);
    }
}

#[test]
fn spec_c02_plan_lineage_resolves_at_repository_paths() {
    require_markers(MVP_PLAN, &[BRAINSTORM]);
    require_markers(RAIL_PLAN, &[BRAINSTORM, MVP_PLAN]);
}

#[test]
fn spec_c03_public_artifacts_exclude_private_path_placeholders() {
    for (path, _) in ARTIFACTS {
        let content = read_repo_file(path);
        assert!(!content.contains("/Users/"), "private user path in {path}");
        assert!(
            !content.contains("/absolute/path/"),
            "absolute path placeholder in {path}"
        );
        require_markers(path, &["not production", "devnet"]);
    }
}

#[test]
fn spec_c04_python_metadata_policy_is_explicit() {
    let ignore = read_repo_file(".gitignore");
    assert!(ignore.lines().any(|line| line == "*.egg-info/"));

    let lockfile = read_repo_file("uv.lock");
    require_content("uv.lock", &lockfile, "version = 1");
}

fn require_markers(path: &str, markers: &[&str]) {
    let content = read_repo_file(path);
    for marker in markers {
        require_content(path, &content, marker);
    }
}

fn require_content(path: &str, content: &str, marker: &str) {
    assert!(content.contains(marker), "{path} is missing `{marker}`");
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("{path} should be readable: {error}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
