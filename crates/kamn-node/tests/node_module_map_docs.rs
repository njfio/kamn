const DOC: &str = include_str!("../../../docs/architecture/kamn-node-module-map.md");

#[test]
fn doc_contains_module_ownership_boundaries() {
    assert!(DOC.contains("# KAMN Node Module Map"));
    assert!(DOC.contains("src/main.rs"));
    assert!(DOC.contains("src/cli.rs"));
    assert!(DOC.contains("src/runtime_kolme_live.rs"));
    assert!(DOC.contains("src/signer.rs"));
    assert!(DOC.contains("src/wire_payload.rs"));
    assert!(DOC.contains("main.rs orchestrates only"));
}

#[test]
fn regression_requires_decomposition_guardrail_markers() {
    // Regression: #2606
    assert!(DOC.contains("Regression: #2606"));
    assert!(DOC.contains("Do not reintroduce parser implementation into src/main.rs"));
}
