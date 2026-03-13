use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn count_lines(path: &Path) -> usize {
    read(path).lines().count()
}

fn block_pipeline_paths(root: &Path) -> (PathBuf, [PathBuf; 5]) {
    let block_pipeline_root = root.join("crates/kamn-core/src/block_pipeline.rs");
    let block_pipeline_dir = root.join("crates/kamn-core/src/block_pipeline");
    (
        block_pipeline_root,
        [
            block_pipeline_dir.join("models.rs"),
            block_pipeline_dir.join("reason_projection.rs"),
            block_pipeline_dir.join("lane_boundary.rs"),
            block_pipeline_dir.join("commit_hooks.rs"),
            block_pipeline_dir.join("tests.rs"),
        ],
    )
}

fn assert_required_modules(paths: &[PathBuf; 5]) {
    for path in paths {
        assert!(path.exists(), "missing {}", path.display());
    }
}

fn assert_root_markers(root_source: &str) {
    for (marker, message) in [
        ("mod models;", "root missing models module marker"),
        (
            "mod reason_projection;",
            "root missing reason_projection module marker",
        ),
        (
            "mod lane_boundary;",
            "root missing lane_boundary module marker",
        ),
        (
            "mod commit_hooks;",
            "root missing commit_hooks module marker",
        ),
        ("#[cfg(test)]", "root missing test module marker"),
        ("mod tests;", "root missing tests module marker"),
    ] {
        assert!(root_source.contains(marker), "{message}");
    }
}

fn assert_budgets(block_pipeline_root: &Path, paths: &[PathBuf; 5]) {
    let block_pipeline_root_lines = count_lines(block_pipeline_root);
    assert!(
        block_pipeline_root_lines <= 180,
        "block pipeline root shell too large: {}",
        block_pipeline_root_lines
    );
    for path in paths {
        let path_lines = count_lines(path);
        assert!(
            path_lines <= 200,
            "{} exceeds file budget with {} lines",
            path.display(),
            path_lines
        );
    }
}

#[test]
fn block_pipeline_root_is_extracted_into_bounded_modules() {
    let root = repo_root();
    let (block_pipeline_root, paths) = block_pipeline_paths(&root);
    assert_required_modules(&paths);
    assert_root_markers(&read(&block_pipeline_root));
    assert_budgets(&block_pipeline_root, &paths);
}
