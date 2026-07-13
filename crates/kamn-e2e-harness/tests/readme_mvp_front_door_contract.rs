use std::path::PathBuf;

#[test]
fn spec_c01_readme_opens_with_current_mvp_front_door() {
    let readme = read_root_readme();

    require_all(
        &readme,
        &[
            "KAMN (Kolme AI Agent Messaging Network)",
            "## What This Repository Contains",
            "## Quickstart",
            "make demo-mvp",
            "cargo run -p kamn-e2e-harness -- verify-mvp-demo",
            "--report .kamn/demo/latest/proof/report.json",
            "docs/validation/mvp-evaluator-demo.md",
            "## Claim Boundaries",
            "local-only",
            "devnet-backed",
            "Solana devnet",
            "developer-test tokens",
            "production readiness",
            "## Architecture Map",
            "## For AI Agents And Maintainers",
        ],
    );

    require_ordered_pairs(
        &readme,
        &[
            ("## What This Repository Contains", "## Quickstart"),
            ("## Quickstart", "## Claim Boundaries"),
            ("## Claim Boundaries", "## Architecture Map"),
            ("## Architecture Map", "## For AI Agents And Maintainers"),
        ],
    );
}

fn read_root_readme() -> String {
    std::fs::read_to_string(root_readme()).expect("root README.md should be readable")
}

fn root_readme() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../README.md")
}

fn require_all(readme: &str, needles: &[&str]) {
    for needle in needles {
        require_contains(readme, needle);
    }
}

fn require_contains(readme: &str, needle: &str) {
    assert!(
        readme.contains(needle),
        "README.md is missing required MVP front-door content: {needle}"
    );
}

fn require_ordered_pairs(readme: &str, pairs: &[(&str, &str)]) {
    for (before, after) in pairs {
        require_order(readme, before, after);
    }
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
