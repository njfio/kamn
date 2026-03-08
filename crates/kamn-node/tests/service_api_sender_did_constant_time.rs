#[test]
fn sender_did_binding_compare_avoids_eq_ignore_ascii_case() {
    let source = std::fs::read_to_string("src/service_api_endpoint/auth.rs")
        .expect("auth source should be readable");
    let function_start = source
        .find("fn sender_did_matches_signer_public_key(")
        .expect("sender DID binding function should exist");
    let function_source = &source[function_start..];

    assert!(
        !function_source.contains(".eq_ignore_ascii_case("),
        "sender DID signer binding compare must not rely on eq_ignore_ascii_case"
    );
}
