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

#[test]
fn signer_root_is_extracted_into_bounded_modules() {
    let root = repo_root();
    let signer_root = root.join("crates/kamn-node/src/signer.rs");
    let signer_dir = root.join("crates/kamn-node/src/signer");
    let models = signer_dir.join("models.rs");
    let secrets = signer_dir.join("secret_provider.rs");
    let managed = signer_dir.join("managed_flow.rs");
    let direct = signer_dir.join("direct_payload.rs");
    let tests = signer_dir.join("tests.rs");

    assert!(models.exists(), "missing {}", models.display());
    assert!(secrets.exists(), "missing {}", secrets.display());
    assert!(managed.exists(), "missing {}", managed.display());
    assert!(direct.exists(), "missing {}", direct.display());
    assert!(tests.exists(), "missing {}", tests.display());

    let root_source = read(&signer_root);
    assert!(root_source.contains("mod models;"), "root missing models module marker");
    assert!(root_source.contains("mod secret_provider;"), "root missing secret_provider module marker");
    assert!(root_source.contains("mod managed_flow;"), "root missing managed_flow module marker");
    assert!(root_source.contains("mod direct_payload;"), "root missing direct_payload module marker");
    assert!(root_source.contains("#[cfg(test)]"), "root missing test module marker");
    assert!(root_source.contains("mod tests;"), "root missing tests module marker");

    assert!(count_lines(&signer_root) <= 180, "signer root shell too large: {}", count_lines(&signer_root));
    for path in [&models, &secrets, &managed, &direct, &tests] {
        assert!(count_lines(path) <= 200, "{} exceeds file budget with {} lines", path.display(), count_lines(path));
    }
}
