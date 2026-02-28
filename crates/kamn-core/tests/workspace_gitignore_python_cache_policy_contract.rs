use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

#[test]
fn workspace_gitignore_declares_python_cache_markers() {
    let gitignore_path = repo_file(".gitignore");
    let gitignore = fs::read_to_string(&gitignore_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", gitignore_path.display(), error));

    for marker in ["__pycache__/", "*.pyc", "*.pyo", "*.pyd"] {
        assert!(
            gitignore.lines().any(|line| line.trim() == marker),
            "missing required python ignore marker `{}` in {}",
            marker,
            gitignore_path.display()
        );
    }
}
