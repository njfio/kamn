use super::super::*;
use crate::signer::KolmeLiveSignerSelection;

pub(super) fn set_env_vars(entries: &[(&'static str, Option<&'static str>)]) -> Vec<EnvVarGuard> {
    entries
        .iter()
        .map(|(name, value)| EnvVarGuard::set(name, *value))
        .collect()
}

pub(super) fn direct_request(suffix: &str) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        format!("op-node-live-{suffix}").as_str(),
        format!("state:node-live-{suffix}").as_str(),
        format!("kamn:did:agent:node-live-{suffix}").as_str(),
        1,
        format!("payload:node-live-{suffix}").as_str(),
    )
    .expect("request should build")
}

pub(super) fn direct_signed_payload(
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    account_id: &str,
) -> (String, KolmeLiveSignerSelection) {
    let reply = format!(r#"{{"next_nonce":{nonce},"account_id":"{account_id}"}}"#);
    let (base_url, _requests) = spawn_kolme_live_mock_server(vec![MockHttpReply::ok(&reply)]);
    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        request,
        None,
        None,
    )
    .expect("signed payload should be produced")
}

pub(super) fn assert_lowercase_hex_128(value: &str, label: &str) {
    assert_eq!(value.len(), 128, "{label} must be 64 bytes hex");
    assert!(
        value
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "{label} must be lowercase hex"
    );
}

pub(super) fn runtime_args(base_url: String, key_source: &str) -> Vec<String> {
    let mut args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        key_source.to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ];
    if key_source == "managed-external" {
        args.extend([
            "--kolme-live-strict-signer-contracts".to_owned(),
            "--kolme-live-signer-profile".to_owned(),
            "ops-primary".to_owned(),
        ]);
    }
    args
}

pub(super) fn local_heavy_probe_inputs() -> Option<(String, String)> {
    if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping signer vector probe; set KAMN_KOLME_LOCAL_HEAVY=1 to run local-heavy parity probe"
        );
        return None;
    }
    let private_key_hex = env::var("KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX")
        .expect("KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX must be set");
    let message = env::var("KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE")
        .expect("KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE must be set");
    Some((private_key_hex, message))
}
