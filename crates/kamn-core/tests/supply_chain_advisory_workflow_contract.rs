use std::path::{Path, PathBuf};

const EXPECTED_TRIVY_ACTION_PIN: &str = "aquasecurity/trivy-action@v0.31.0";
const LEGACY_UNPREFIXED_TRIVY_PIN: &str = "aquasecurity/trivy-action@0.28.0";
const LEGACY_SETUP_UNSAFE_TRIVY_PIN: &str = "aquasecurity/trivy-action@v0.28.0";

#[test]
fn supply_chain_advisory_uses_setup_safe_trivy_action_pin() {
    let workflow = std::fs::read_to_string(workflow_path())
        .expect("supply-chain advisory workflow should be readable");

    assert!(
        workflow.contains(EXPECTED_TRIVY_ACTION_PIN),
        "expected workflow to pin Trivy action as {EXPECTED_TRIVY_ACTION_PIN}"
    );
    assert!(
        !workflow.contains(LEGACY_UNPREFIXED_TRIVY_PIN),
        "legacy unprefixed Trivy action pin must not return"
    );
    assert!(
        !workflow.contains(LEGACY_SETUP_UNSAFE_TRIVY_PIN),
        "legacy setup-unsafe Trivy action pin must not return"
    );
}

fn workflow_path() -> PathBuf {
    repo_root().join(".github/workflows/ci-supply-chain-advisory.yml")
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("kamn-core manifest should live under crates/kamn-core")
}
