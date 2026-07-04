const SIGNATURE_PROFILE_ENCODING_SOURCE: &str =
    include_str!("../src/signature_profile/encoding.rs");
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
    service_auth_verify_with_public_key_hex,
};

fn function_body<'a>(source: &'a str, fn_name: &str) -> Option<&'a str> {
    let fn_start = source.find(fn_name)?;
    let brace_open = source[fn_start..].find('{')? + fn_start + 1;
    let mut depth = 1_usize;
    for (offset, ch) in source[brace_open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let brace_close = brace_open + offset;
                    return Some(&source[brace_open..brace_close]);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn regression_issue_5924_signature_profile_wipe_bytes_uses_zeroize_trait() {
    // Regression: #5924
    let wipe_bytes_body = function_body(SIGNATURE_PROFILE_ENCODING_SOURCE, "fn wipe_bytes")
        .expect("wipe_bytes function must remain present");
    assert!(
        wipe_bytes_body.contains("bytes.zeroize();"),
        "wipe_bytes must use zeroize-backed erasure"
    );
    assert!(
        !wipe_bytes_body.contains("for "),
        "manual loops must not be reintroduced in wipe_bytes"
    );
}

#[test]
fn integration_issue_5924_service_auth_round_trip_remains_valid() {
    // Regression: #5924
    let public_key =
        service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_AUTH_PRIVATE_KEY_HEX)
            .expect("public key should derive from valid private key");
    let payload = "{\"message\":\"zeroize-regression\"}";
    let signature = service_auth_sign_with_private_key_hex(
        "agent-5924",
        5924,
        "service-api:chain:5924",
        payload,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("service auth signature should render");
    service_auth_verify_with_public_key_hex(
        signature.as_str(),
        "agent-5924",
        5924,
        "service-api:chain:5924",
        payload,
        public_key.as_str(),
    )
    .expect("signature verification should remain stable after wipe-bytes refactor");
}
