use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate dir should have workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn data_layer_m2_gateway_access_root_is_extracted() {
    let root = repo_root();
    let source = root.join("crates/kamn-core/src/data_layer_m2_gateway_access.rs");
    let source_text = read(&source);

    let expected_root_markers = [
        "mod auth;",
        "mod authorization;",
        "mod rls;",
        "mod audit;",
        "mod models;",
        "#[cfg(test)]",
        "mod tests;",
    ];
    for marker in expected_root_markers {
        assert!(
            source_text.contains(marker),
            "expected root shell marker `{marker}` in {}",
            source.display()
        );
    }

    let forbidden_root_markers = [
        "pub struct DataLayerM2DidAuthRequest",
        "pub struct DataLayerM2DidSessionService",
        "pub enum DataLayerM2ActorRole",
        "pub fn data_layer_m2_default_rls_policies()",
        "pub struct DataLayerM2AccessAuditInput",
        "mod tests {",
    ];
    for marker in forbidden_root_markers {
        assert!(
            !source_text.contains(marker),
            "root shell should not keep inline marker `{marker}` in {}",
            source.display()
        );
    }

    assert!(
        source_text.lines().count() <= 180,
        "root shell must stay within staged cap: {} lines",
        source_text.lines().count()
    );

    let expected_modules = [
        "crates/kamn-core/src/data_layer_m2_gateway_access/auth.rs",
        "crates/kamn-core/src/data_layer_m2_gateway_access/authorization.rs",
        "crates/kamn-core/src/data_layer_m2_gateway_access/rls.rs",
        "crates/kamn-core/src/data_layer_m2_gateway_access/audit.rs",
        "crates/kamn-core/src/data_layer_m2_gateway_access/models.rs",
        "crates/kamn-core/src/data_layer_m2_gateway_access/tests.rs",
    ];

    for relative in expected_modules {
        let module_path = root.join(relative);
        assert!(module_path.exists(), "expected extracted module {}", module_path.display());
        let module_text = read(&module_path);
        assert!(
            module_text.lines().count() <= 200,
            "extracted module {} exceeds 200 LOC: {}",
            module_path.display(),
            module_text.lines().count()
        );
    }
}
