const MANAGED_BACKEND_RESPONSE_SRC: &str =
    include_str!("../src/signer/managed_backend/response.rs");
const MANAGED_BACKEND_VERIFICATION_SRC: &str =
    include_str!("../src/signer/managed_backend/response/verification.rs");

#[test]
fn regression_requires_constant_time_managed_signer_public_key_provenance_compare() {
    assert!(
        MANAGED_BACKEND_RESPONSE_SRC.contains(
            "verify_public_key_match(expected_signer_public_key_hex, backend_signature)?;"
        ),
        "managed signer provenance entrypoint must delegate public key comparison"
    );
    let function_start = MANAGED_BACKEND_VERIFICATION_SRC
        .find("fn verify_public_key_match(")
        .unwrap_or_else(|| panic!("managed signer public-key provenance helper must exist"));
    let function_source = &MANAGED_BACKEND_VERIFICATION_SRC[function_start..];

    assert!(
        !function_source.contains(".eq_ignore_ascii_case("),
        "managed signer provenance compare must not rely on eq_ignore_ascii_case"
    );
    assert!(
        function_source.contains("constant_time_eq"),
        "managed signer provenance helper must use a constant-time compare helper"
    );
}
