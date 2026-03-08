const MANAGED_BACKEND_SRC: &str = include_str!("../src/signer/managed_backend.rs");

#[test]
fn regression_requires_constant_time_managed_signer_public_key_provenance_compare() {
    let function_start = MANAGED_BACKEND_SRC
        .find("fn verify_kolme_live_managed_signer_backend_signature_provenance(")
        .unwrap_or_else(|| panic!("managed signer provenance helper must exist"));
    let function_source = &MANAGED_BACKEND_SRC[function_start..];

    assert!(
        !function_source.contains(".eq_ignore_ascii_case("),
        "managed signer provenance compare must not rely on eq_ignore_ascii_case"
    );
    assert!(
        function_source.contains("constant_time_eq"),
        "managed signer provenance helper must use a constant-time compare helper"
    );
}
