const DOC: &str = include_str!("../../../docs/foundation/kolme-runtime-architecture.md");
const README: &str = include_str!("../../../README.md");

#[test]
fn architecture_doc_contains_runtime_flow_and_signer_boundaries() {
    assert!(DOC.contains("## Runtime Flow Diagram"));
    assert!(DOC.contains("```mermaid"));
    assert!(DOC.contains("graph TD"));
    assert!(DOC.contains("kamn-node"));
    assert!(DOC.contains("kamn-core"));
    assert!(DOC.contains("kamn-kolme"));
    assert!(DOC.contains("KolmeRuntimeCommitLiveProvider"));
    assert!(DOC.contains("managed-external signer backend"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"));
    assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"));
}

#[test]
fn architecture_doc_contains_module_ownership_map() {
    assert!(DOC.contains("## Ownership Map"));
    assert!(DOC.contains("crates/kamn-node/src/runtime_kolme_live.rs"));
    assert!(DOC.contains("crates/kamn-node/src/signer.rs"));
    assert!(DOC.contains("crates/kamn-core/src/kolme_runtime_commit.rs"));
    assert!(DOC.contains("crates/kamn-kolme/src/live_provider_pipeline.rs"));
}

#[test]
fn readme_references_architecture_doc() {
    assert!(README.contains("docs/foundation/kolme-runtime-architecture.md"));
    assert!(README.contains("docs/foundation/kolme-runtime-architecture.md#runtime-flow-diagram"));
    assert!(README.contains("docs/foundation/kolme-runtime-architecture.md#ownership-map"));
}
