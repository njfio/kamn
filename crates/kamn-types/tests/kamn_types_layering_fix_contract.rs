use std::fs;
use std::path::PathBuf;

const README: &str = include_str!("../README.md");
const ARCH_DOC: &str = include_str!("../../../docs/architecture/kamn-types.md");

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn kamn_types_cargo_toml_does_not_depend_on_kamn_core() {
    let cargo_toml = repo_file("crates/kamn-types/Cargo.toml");
    assert!(
        !cargo_toml.contains("kamn-core = { path = \"../kamn-core\" }"),
        "kamn-types must not depend on kamn-core once the layering fix lands"
    );
}

#[test]
fn kamn_types_source_owns_first_wave_did_surface() {
    let source = repo_file("crates/kamn-types/src/lib.rs");
    assert!(
        !source.contains("pub use kamn_core::"),
        "kamn-types should own the first-wave DID surface instead of re-exporting it from kamn-core"
    );
    assert!(
        !source.contains("pub use kamn_core::{"),
        "kamn-types should not forward grouped DID exports from kamn-core"
    );
}

#[test]
fn kamn_types_docs_report_post_inversion_state() {
    assert!(
        README.contains("kamn_types_dependency_status=owned-did-surface"),
        "README must describe the post-inversion dependency state"
    );
    assert!(
        ARCH_DOC.contains("kamn_types_current_dependency_status=owned-did-surface"),
        "architecture doc must no longer advertise the temporary kamn-core re-export state"
    );
    assert!(
        ARCH_DOC.contains("kamn_types_target_dependency_policy=no-kamn-core"),
        "architecture doc must continue to state the no-kamn-core target dependency policy"
    );
}
