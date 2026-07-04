#[test]
fn sender_did_binding_compare_avoids_eq_ignore_ascii_case() {
    let binding_source =
        std::fs::read_to_string("src/service_api_endpoint/auth/request_auth/sender_binding.rs")
            .expect("sender binding source should be readable");
    let support_source = std::fs::read_to_string("src/service_api_endpoint/auth/support.rs")
        .expect("auth support source should be readable");
    let function_start = binding_source
        .find("fn sender_did_matches_signer_public_key(")
        .expect("sender DID binding function should exist");
    let function_source = &binding_source[function_start..];

    assert!(
        !function_source.contains(".eq_ignore_ascii_case("),
        "sender DID signer binding compare must not rely on eq_ignore_ascii_case"
    );
    assert!(
        function_source.contains("normalized_public_key_hexes_match("),
        "sender DID signer binding compare should delegate public-key equality to the normalized helper"
    );
    assert!(
        support_source.contains("fn constant_time_eq_bytes("),
        "normalized public-key equality helper should use a local constant-time byte comparison"
    );
}
