const ADR: &str = include_str!("../../../docs/architecture/adr-kamn-core-live-tls-transport.md");
const README: &str = include_str!("../../../README.md");
const RUNTIME_COMMIT_DOC: &str =
    include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");
const OPS_CONFIGURATION_DOC: &str = include_str!("../../../docs/ops/configuration.md");
const TLS_HARDENING_DOC: &str = include_str!("../../../docs/security/tls-hardening.md");

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

#[test]
fn ops_configuration_doc_tracks_tls_runtime_transport_behavior_contracts() {
    assert!(OPS_CONFIGURATION_DOC.contains("KAMN_KOLME_TLS_CA_FILE"));
    assert!(OPS_CONFIGURATION_DOC.contains("tls certificate verification failed"));
    assert!(OPS_CONFIGURATION_DOC.contains("tls handshake failed"));
    assert!(OPS_CONFIGURATION_DOC.contains("in-process rustls"));
    assert!(OPS_CONFIGURATION_DOC.contains("Subprocess fallback is not allowed"));
    assert!(OPS_CONFIGURATION_DOC.contains("Regression: #4106"));
}

#[test]
fn security_tls_hardening_doc_tracks_reason_class_marker() {
    assert!(TLS_HARDENING_DOC.contains("reason_class=stable|violation"));
}
