use k256::sha2::{Digest, Sha256};
use kamn_core::{
    data_layer_m3_compute_blind_index, DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE,
};

const DATA_LAYER_M0_SOURCE: &str = include_str!("../src/data_layer_m0.rs");
const DATA_LAYER_M1_SOURCE: &str = include_str!("../src/data_layer_m1.rs");
const DATA_LAYER_M2_SOURCE: &str = include_str!("../src/data_layer_m2_gateway_access.rs");
const DATA_LAYER_M3_SOURCE: &str = include_str!("../src/data_layer_m3_blind_index_search.rs");
const DATA_LAYER_M4_SOURCE: &str = include_str!("../src/data_layer_m4_escrow_integration.rs");
const DATA_LAYER_M5_SOURCE: &str = include_str!("../src/data_layer_m5_vector_integration.rs");

fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn regression_issue_5922_m3_blind_index_digest_matches_real_sha256_vector() {
    // Regression: #5922
    let token = data_layer_m3_compute_blind_index("pepper-5922", "subject", "Alice Example")
        .expect("blind index derivation should succeed");
    let expected_payload = format!(
        "m3-blind-index|key:pepper-5922|field:subject|value:alice example|profile:{}",
        DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE
    );
    let expected = format!("sha256:{}", sha256_hex(expected_payload.as_str()));
    assert_eq!(token, expected);
}

#[test]
fn regression_issue_5922_m0_m5_remove_custom_digest_mixers() {
    // Regression: #5922
    let modules = [
        ("m0", DATA_LAYER_M0_SOURCE),
        ("m1", DATA_LAYER_M1_SOURCE),
        ("m2", DATA_LAYER_M2_SOURCE),
        ("m3", DATA_LAYER_M3_SOURCE),
        ("m4", DATA_LAYER_M4_SOURCE),
        ("m5", DATA_LAYER_M5_SOURCE),
    ];

    for (module, source) in modules {
        assert!(
            !source.contains("fn deterministic_digest_256_hex"),
            "{module} must not define deterministic_digest_256_hex"
        );
        assert!(
            source.contains("tagged_sha256("),
            "{module} must route tagged digesting through shared tagged_sha256 helper"
        );
    }
}
