const SIGNER_RS: &str = include_str!("../src/signer.rs");
const SIGNER_ADAPTER_RS: &str = include_str!("../src/signer/signer_adapter.rs");
const KOLME_RUNTIME_COMMIT_DOC: &str =
    include_str!("../../../docs/architecture/kolme-runtime-commit.md");

const REQUIRED_ADAPTER_EXPORTS: &[&str] = &[
    "build_kolme_live_managed_signing_key",
    "decode_kolme_hex_bytes",
    "encode_kolme_hex_lower",
    "resolve_kolme_live_signer_private_key_env_name",
    "KolmeForkSecp256k1SignerAdapter",
];

#[test]
fn source_declares_signer_adapter_boundary_re_exports() {
    assert!(
        SIGNER_RS.contains("mod signer_adapter;"),
        "signer.rs must declare signer_adapter module"
    );
    assert!(
        SIGNER_RS.contains("pub(crate) use signer_adapter::{"),
        "signer.rs must re-export signer_adapter owned API"
    );
    for symbol in REQUIRED_ADAPTER_EXPORTS {
        assert!(
            SIGNER_RS.contains(symbol),
            "signer.rs re-export surface missing signer_adapter symbol: {symbol}"
        );
    }
}

#[test]
fn source_enforces_signer_adapter_ownership_without_reinline_backslide() {
    assert!(
        SIGNER_ADAPTER_RS.contains("pub(crate) struct KolmeForkSecp256k1SignerAdapter"),
        "signer_adapter.rs must own signer adapter struct"
    );
    assert!(
        SIGNER_ADAPTER_RS.contains("fn decode_kolme_hex_nibble("),
        "signer_adapter.rs must own hex nibble decoder"
    );
    assert!(
        SIGNER_ADAPTER_RS.contains("pub(crate) fn decode_kolme_hex_bytes("),
        "signer_adapter.rs must own hex decode helper"
    );
    assert!(
        SIGNER_ADAPTER_RS.contains("pub(crate) fn encode_kolme_hex_lower("),
        "signer_adapter.rs must own hex encode helper"
    );

    assert!(
        !SIGNER_RS.contains("fn decode_kolme_hex_nibble("),
        "signer.rs must not re-inline hex nibble decoder"
    );
    assert!(
        !SIGNER_RS.contains("fn decode_kolme_hex_bytes("),
        "signer.rs must not re-inline hex decode helper"
    );
    assert!(
        !SIGNER_RS.contains("pub(crate) fn encode_kolme_hex_lower("),
        "signer.rs must not re-inline hex encode helper"
    );
    assert!(
        !SIGNER_RS.contains("impl KolmeForkSecp256k1SignerAdapter {"),
        "signer.rs must not re-inline signer adapter implementation"
    );
}

#[test]
fn docs_declare_signer_adapter_boundary_markers() {
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains("### Signer Adapter API Boundary"),
        "runtime-commit docs must declare signer adapter API boundary section"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains("signer_adapter_boundary_contract_status=active"),
        "runtime-commit docs must declare signer adapter boundary status marker"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains("signer_adapter_boundary_contract_version=v1"),
        "runtime-commit docs must declare signer adapter boundary version marker"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains(
            "signer_adapter_owned_symbols_csv=KolmeForkSecp256k1SignerAdapter,decode_kolme_hex_bytes,encode_kolme_hex_lower,build_kolme_live_managed_signing_key,resolve_kolme_live_signer_private_key_env_name"
        ),
        "runtime-commit docs must declare signer_adapter owned symbol marker"
    );
    assert!(
        KOLME_RUNTIME_COMMIT_DOC.contains(
            "cargo test -p kamn-node --test signer_adapter_boundary_contract -- --nocapture"
        ),
        "runtime-commit docs must declare signer adapter boundary contract command"
    );
}
