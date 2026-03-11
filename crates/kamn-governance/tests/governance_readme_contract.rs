use std::fs;
use std::path::PathBuf;

fn governance_readme() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(manifest_dir.join("README.md"))
        .expect("governance README must exist for crate onboarding")
}

#[test]
fn governance_readme_contains_required_markers() {
    let readme = governance_readme();
    for marker in [
        "# kamn-governance",
        "## Purpose",
        "## Exported Surfaces",
        "## Local Verification",
    ] {
        assert!(
            readme.contains(marker),
            "README missing required marker: {marker}"
        );
    }
}
