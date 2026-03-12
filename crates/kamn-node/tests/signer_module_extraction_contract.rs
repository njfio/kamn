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
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn count_lines(path: &Path) -> usize {
    read(path).lines().count()
}

fn signer_paths(root: &Path) -> (PathBuf, [PathBuf; 5]) {
    let signer_root = root.join("crates/kamn-node/src/signer.rs");
    let signer_dir = root.join("crates/kamn-node/src/signer");
    (
        signer_root,
        [
            signer_dir.join("models.rs"),
            signer_dir.join("secret_provider.rs"),
            signer_dir.join("managed_flow.rs"),
            signer_dir.join("direct_payload.rs"),
            signer_dir.join("tests.rs"),
        ],
    )
}

fn assert_required_modules(paths: &[PathBuf; 5]) {
    for path in paths {
        assert!(path.exists(), "missing {}", path.display());
    }
}

fn assert_root_markers(root_source: &str) {
    assert!(root_source.contains("mod models;"), "root missing models module marker");
    assert!(
        root_source.contains("mod secret_provider;"),
        "root missing secret_provider module marker"
    );
    assert!(
        root_source.contains("mod managed_flow;"),
        "root missing managed_flow module marker"
    );
    assert!(
        root_source.contains("mod direct_payload;"),
        "root missing direct_payload module marker"
    );
    assert!(root_source.contains("#[cfg(test)]"), "root missing test module marker");
    assert!(root_source.contains("mod tests;"), "root missing tests module marker");
}

fn assert_budgets(signer_root: &Path, paths: &[PathBuf; 5]) {
    assert!(
        count_lines(signer_root) <= 180,
        "signer root shell too large: {}",
        count_lines(signer_root)
    );
    for path in paths {
        assert!(
            count_lines(path) <= 200,
            "{} exceeds file budget with {} lines",
            path.display(),
            count_lines(path)
        );
    }
}

#[test]
fn signer_root_is_extracted_into_bounded_modules() {
    let root = repo_root();
    let (signer_root, paths) = signer_paths(&root);
    assert_required_modules(&paths);
    assert_root_markers(&read(&signer_root));
    assert_budgets(&signer_root, &paths);
}
