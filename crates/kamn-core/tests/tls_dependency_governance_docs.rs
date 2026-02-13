const ADR: &str = include_str!("../../../docs/architecture/adr-kamn-core-live-tls-transport.md");
const README: &str = include_str!("../../../README.md");
const RUNTIME_COMMIT_DOC: &str =
    include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");

#[test]
fn adr_documents_live_tls_dependency_decision_and_tradeoffs() {
    assert!(ADR.contains("`kamn-core` Live TLS Transport Dependency Posture"));
    assert!(ADR.contains("rustls"));
    assert!(ADR.contains("rustls-pemfile"));
    assert!(ADR.contains("webpki-roots"));
    assert!(ADR.contains("Subprocess TLS paths (`curl`, `openssl s_client`) are not allowed"));
    assert!(ADR.contains("Compile-time feature gate for local-only builds"));
    assert!(ADR.contains("`live-https` default-on"));
    assert!(ADR.contains("--no-default-features"));
    assert!(ADR.contains("crates/kamn-core/src/kolme_runtime_commit/http_transport.rs"));
    assert!(ADR.contains("crates/kamn-kolme/src/tls_policy.rs"));
    assert!(ADR.contains("crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs"));
}

#[test]
fn readme_and_foundation_transport_doc_reference_tls_dependency_adr() {
    assert!(README.contains("docs/architecture/adr-kamn-core-live-tls-transport.md"));
    assert!(RUNTIME_COMMIT_DOC.contains("docs/architecture/adr-kamn-core-live-tls-transport.md"));
}

#[test]
fn runtime_commit_transport_doc_keeps_in_process_tls_narrative() {
    assert!(RUNTIME_COMMIT_DOC.contains("in-process `rustls` transport wiring"));
    assert!(!RUNTIME_COMMIT_DOC.contains("openssl s_client command wiring"));
}
