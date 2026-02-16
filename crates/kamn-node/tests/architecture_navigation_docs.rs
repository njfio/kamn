const ARCH_NAV_INDEX: &str = include_str!("../../../docs/architecture/README.md");
const REPO_README: &str = include_str!("../../../README.md");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn architecture_navigation_index_declares_schema_and_diagram_catalog_markers() {
    assert!(ARCH_NAV_INDEX.contains("schema_version=kamn.docs.architecture-navigation-index.v1"));
    assert!(ARCH_NAV_INDEX.contains("diagram_catalog_status=active"));
    assert!(ARCH_NAV_INDEX.contains("diagram:runtime-layout"));
    assert!(ARCH_NAV_INDEX.contains("diagram:service-runtime"));
    assert!(ARCH_NAV_INDEX.contains("diagram:block-pipeline"));
    assert!(ARCH_NAV_INDEX.contains("diagram:p2p-transport"));
    assert!(ARCH_NAV_INDEX.contains("diagram:kolme-live-integration"));
    assert!(ARCH_NAV_INDEX.contains("diagram:signer-lifecycle"));
}

#[test]
fn architecture_navigation_index_links_required_artifacts() {
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/kamn-core-module-map.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/kamn-node-module-map.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/runtime-layout.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/service-runtime.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/block-pipeline.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/p2p-transport.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/kolme-live-integration.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/kolme-runtime-commit.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/persistence-backends.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/did-chain-adapter.md"));
    assert!(ARCH_NAV_INDEX.contains("docs/architecture/adr-kamn-core-live-tls-transport.md"));
}

#[test]
fn readme_and_ci_strategy_reference_architecture_navigation_guard() {
    assert!(REPO_README.contains("docs/architecture/README.md"));
    assert!(CI_STRATEGY_DOC.contains("architecture navigation index guard"));
    assert!(CI_STRATEGY_DOC.contains("cargo test -p kamn-node --test architecture_navigation_docs"));
}
