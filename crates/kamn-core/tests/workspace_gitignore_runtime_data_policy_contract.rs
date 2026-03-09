use std::fs;
use std::path::PathBuf;

const REQUIRED_RUNTIME_DATA_IGNORE_MARKERS: [&str; 2] = ["data/", "crates/kamn-node/data/"];

fn repo_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

#[test]
fn workspace_gitignore_declares_runtime_data_markers() {
    let gitignore_path = repo_file(".gitignore");
    let gitignore = fs::read_to_string(&gitignore_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", gitignore_path.display(), error));

    for marker in REQUIRED_RUNTIME_DATA_IGNORE_MARKERS {
        assert!(
            gitignore.lines().any(|line| line.trim() == marker),
            "missing required runtime-data ignore marker `{}` in {}",
            marker,
            gitignore_path.display()
        );
    }
}
