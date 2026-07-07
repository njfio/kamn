use std::path::PathBuf;

#[test]
fn spec_c01_readme_opens_with_current_mvp_front_door() {
    let readme = read_root_readme();

    require_contains(&readme, "KAMN (Kolme AI Agent Messaging Network)");
    require_contains(&readme, "## What KAMN Proves Today");
    require_contains(&readme, "## MVP Demo Quickstart");
    require_contains(&readme, "make demo-mvp");
    require_contains(
        &readme,
        "cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json",
    );
    require_contains(&readme, "docs/validation/mvp-evaluator-demo.md");
    require_contains(&readme, "## Claim Boundaries");
    require_contains(&readme, "local-only");
    require_contains(&readme, "devnet-backed");
    require_contains(&readme, "Solana devnet");
    require_contains(&readme, "developer-test tokens");
    require_contains(&readme, "production readiness");
    require_contains(&readme, "## Repository Map");
    require_contains(&readme, "## For AI Agents And Maintainers");

    require_order(
        &readme,
        "## What KAMN Proves Today",
        "## MVP Demo Quickstart",
    );
    require_order(&readme, "## MVP Demo Quickstart", "## Claim Boundaries");
    require_order(&readme, "## Claim Boundaries", "## Repository Map");
    require_order(
        &readme,
        "## Repository Map",
        "## For AI Agents And Maintainers",
    );
}

fn read_root_readme() -> String {
    std::fs::read_to_string(root_readme()).expect("root README.md should be readable")
}

fn root_readme() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../README.md")
}

fn require_contains(readme: &str, needle: &str) {
    assert!(
        readme.contains(needle),
        "README.md is missing required MVP front-door content: {needle}"
    );
}

fn require_order(readme: &str, before: &str, after: &str) {
    let before_index = readme
        .find(before)
        .unwrap_or_else(|| panic!("README.md is missing required heading: {before}"));
    let after_index = readme
        .find(after)
        .unwrap_or_else(|| panic!("README.md is missing required heading: {after}"));
    assert!(
        before_index < after_index,
        "README.md should place `{before}` before `{after}`"
    );
}
