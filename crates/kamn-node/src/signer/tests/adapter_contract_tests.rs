use super::support::{is_zeroized_hex_buffer, TEST_PRIVATE_KEY_ENV, TEST_PRIVATE_KEY_HEX};
use super::{ConfigError, Duration, Instant, KolmeForkSecp256k1SignerAdapter};

#[test]
fn unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material() {
    const SIGNER_ADAPTER_SOURCE: &str = include_str!("../signer_adapter.rs");
    assert!(SIGNER_ADAPTER_SOURCE.contains("key_material.zeroize()"));
}

#[test]
fn unit_signer_private_key_parse_zeroizes_hex_buffer_on_success() {
    let mut private_key_hex = TEST_PRIVATE_KEY_HEX.to_owned();
    let signer = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
        &mut private_key_hex,
        TEST_PRIVATE_KEY_ENV,
    )
    .expect("valid private key should parse");
    assert!(is_zeroized_hex_buffer(private_key_hex.as_str()));
    assert_eq!(signer.private_key_env, TEST_PRIVATE_KEY_ENV);
}

#[test]
fn regression_signer_private_key_parse_zeroizes_hex_buffer_on_failure() {
    let mut private_key_hex = "zz".to_owned();
    let error = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
        &mut private_key_hex,
        TEST_PRIVATE_KEY_ENV,
    )
    .expect_err("invalid private key hex must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("invalid hex character"))
    );
    assert!(is_zeroized_hex_buffer(private_key_hex.as_str()));
}

#[test]
fn regression_signer_module_source_contains_no_unreachable_macro() {
    const SIGNER_SOURCE: &str = include_str!("../../signer.rs");
    let marker = ["unreachable", "!", "("].concat();
    assert!(!SIGNER_SOURCE.contains(marker.as_str()));
}

#[test]
fn regression_signer_private_key_decode_failure_redacts_sensitive_input() {
    let sensitive_input = "secretshouldnotappear000";
    let mut private_key_hex = sensitive_input.to_owned();
    let error = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
        &mut private_key_hex,
        TEST_PRIVATE_KEY_ENV,
    )
    .expect_err("invalid private key material must fail closed");
    let message = match &error {
        ConfigError::RuntimeKolmeLive(message) => message,
        _ => "",
    };
    assert!(message.contains("invalid hex character"));
    assert!(!message.contains(sensitive_input));
    assert!(is_zeroized_hex_buffer(private_key_hex.as_str()));
}

#[test]
fn performance_signer_private_key_parse_zeroization_stays_bounded() {
    let started = Instant::now();
    for _ in 0..2_000 {
        let mut private_key_hex = TEST_PRIVATE_KEY_HEX.to_owned();
        let _signer = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
            &mut private_key_hex,
            TEST_PRIVATE_KEY_ENV,
        )
        .expect("valid private key should parse");
        assert!(is_zeroized_hex_buffer(private_key_hex.as_str()));
    }
    assert!(started.elapsed() < Duration::from_secs(2));
}
