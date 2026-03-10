use super::super::*;
use super::support::{
    assert_lowercase_hex_128, direct_request, direct_signed_payload, local_heavy_probe_inputs,
    set_env_vars,
};
use crate::signer::KolmeLiveSignerSelection;

const OPS_PRIMARY_ENV: &[(&str, Option<&str>)] = &[
    ("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary")),
    (
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX),
    ),
];

#[test]
fn unit_kolme_live_signer_builds_direct_signed_wire_payload() {
    let _lock = lock_signer_env_guard();
    let _env = set_env_vars(OPS_PRIMARY_ENV);
    let request = direct_request("2197");
    let (payload, selection) = direct_signed_payload(&request, 22, "acct-2197");
    assert_direct_selection(&selection);
    assert_payload_signature(payload.as_str());
}

#[test]
fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message() {
    let _lock = lock_signer_env_guard();
    let _env = set_env_vars(OPS_PRIMARY_ENV);
    let message = direct_adapter_message();
    let (adapter, selection) =
        build_kolme_live_signer_adapter(None, None).expect("adapter should build");
    assert_eq!(selection.profile, "ops-primary");
    let (signature_hex, recovery_id) = adapter
        .sign_message(message)
        .expect("adapter signing should succeed");
    assert_lowercase_hex_128(signature_hex.as_str(), "adapter signature");
    adapter
        .verify_message(message, signature_hex.as_str(), recovery_id)
        .expect("adapter signature verification should succeed");
}

#[test]
fn integration_kolme_live_signer_vector_probe_contract() {
    let Some((private_key_hex, message)) = local_heavy_probe_inputs() else {
        return;
    };
    let adapter = build_vector_probe_adapter(private_key_hex.as_str());
    let (signature_hex, recovery_id) = adapter
        .sign_message(message.as_str())
        .expect("signature parity adapter signing should succeed");
    let pubkey_hex = adapter.public_key_compressed_hex();
    print_vector_probe(signature_hex.as_str(), recovery_id, pubkey_hex.as_str());
    assert_expected_vector_values(signature_hex.as_str(), recovery_id, pubkey_hex.as_str());
}

fn assert_direct_selection(selection: &KolmeLiveSignerSelection) {
    assert_eq!(selection.profile, "ops-primary");
    assert_eq!(
        selection.private_key_env,
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"
    );
    assert_eq!(selection.key_source, "env-local");
}

fn assert_payload_signature(payload: &str) {
    assert!(payload.contains("\"message\":\"{\\\"pubkey\\\":"));
    let signature = extract_json_string_field(payload, "signature")
        .expect("direct signed payload must include signature field");
    assert_lowercase_hex_128(signature.as_str(), "direct payload signature");
}

fn direct_adapter_message() -> &'static str {
    "{\"pubkey\":\"pk-adapter\",\"nonce\":7,\"created\":\"2026-02-12T00:00:00Z\",\"messages\":[]}"
}

fn build_vector_probe_adapter(private_key_hex: &str) -> KolmeForkSecp256k1SignerAdapter {
    KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        private_key_hex,
        "KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX",
    )
    .expect("signature parity adapter should build")
}

fn print_vector_probe(signature_hex: &str, recovery_id: u8, pubkey_hex: &str) {
    println!("signature_hex={signature_hex}");
    println!("recovery_id={recovery_id}");
    println!("pubkey_hex={pubkey_hex}");
}

fn assert_expected_vector_values(signature_hex: &str, recovery_id: u8, pubkey_hex: &str) {
    assert_expected_signature(signature_hex);
    assert_expected_recovery_id(recovery_id);
    assert_expected_pubkey(pubkey_hex);
}

fn assert_expected_signature(signature_hex: &str) {
    if let Ok(expected_signature_hex) =
        env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_SIGNATURE_HEX")
    {
        assert_eq!(
            signature_hex, expected_signature_hex,
            "signature parity probe must match expected signature vector"
        );
    }
}

fn assert_expected_recovery_id(recovery_id: u8) {
    if let Ok(expected_recovery_id) = env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_RECOVERY_ID") {
        let expected_recovery_id = expected_recovery_id
            .parse::<u8>()
            .expect("expected recovery id must parse as u8");
        assert_eq!(
            recovery_id, expected_recovery_id,
            "signature parity probe must match expected recovery id vector"
        );
    }
}

fn assert_expected_pubkey(pubkey_hex: &str) {
    if let Ok(expected_pubkey_hex) = env::var("KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_PUBKEY_HEX") {
        assert_eq!(
            pubkey_hex, expected_pubkey_hex,
            "signature parity probe must match expected pubkey vector"
        );
    }
}
