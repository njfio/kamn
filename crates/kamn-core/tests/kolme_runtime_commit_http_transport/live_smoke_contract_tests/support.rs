use super::*;

pub(crate) fn local_heavy_enabled() -> bool {
    if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() == Some("1") {
        return true;
    }
    eprintln!(
        "skipping live-node smoke; set KAMN_KOLME_LOCAL_HEAVY=1 to run local-heavy live probe"
    );
    false
}

pub(crate) fn required_live_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} must be set for live node smoke"))
}

pub(crate) fn optional_live_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

pub(crate) fn assert_live_signing_profile() {
    let profile = optional_live_env(KOLME_LIVE_SIGNING_PROFILE_ENV, KOLME_FORK_SECP256K1_PROFILE);
    assert_eq!(profile, KOLME_FORK_SECP256K1_PROFILE);
}

pub(crate) fn live_smoke_provider(
    base_url: &str,
    provider_hint: &str,
) -> KolmeRuntimeCommitLiveProvider<KolmeRuntimeCommitHttpTransport> {
    let transport = live_smoke_transport();
    KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url,
        provider_hint,
        transport,
    )
    .expect("provider should build")
}

fn live_smoke_transport() -> KolmeRuntimeCommitHttpTransport {
    match env::var("KAMN_KOLME_LIVE_AUTHORIZATION").ok() {
        Some(value) => KolmeRuntimeCommitHttpTransport::new_with_authorization(10, value.as_str())
            .expect("transport with authorization should build"),
        None => KolmeRuntimeCommitHttpTransport::new(10).expect("transport should build"),
    }
}

pub(crate) fn live_smoke_payload() -> (String, String) {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let signer_pubkey = kolme_fork_live_smoke_pubkey_hex();
    let nonce = ((unique_suffix % 1_000_000_000_u128) as u64) + 1;
    let message_json = live_smoke_message_json(signer_pubkey.as_str(), nonce);
    let (signature, recovery_id) = kolme_fork_sign_message(message_json.as_str());
    (
        format!(
            "{{\"message\":\"{}\",\"signature\":\"{}\",\"recovery_id\":{}}}",
            message_json.replace('\\', "\\\\").replace('"', "\\\""),
            signature,
            recovery_id
        ),
        format!("kolme-runtime-commit:live-smoke:{unique_suffix}"),
    )
}

pub(crate) fn live_smoke_message_json(pubkey: &str, nonce: u64) -> String {
    format!(
        "{{\"pubkey\":\"{pubkey}\",\"nonce\":{nonce},\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}}"
    )
}

pub(crate) fn assert_live_reachability_outcome(
    outcome: Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>,
) {
    match outcome {
        Ok(KolmeRuntimeCommitProviderOutcome::Submitted(receipt))
        | Ok(KolmeRuntimeCommitProviderOutcome::Duplicate(receipt)) => {
            assert!(!receipt.provider.trim().is_empty());
            assert!(!receipt.commit_id.trim().is_empty());
        }
        Ok(KolmeRuntimeCommitProviderOutcome::Rejected { reason }) => {
            assert!(!reason.trim().is_empty());
        }
        Err(KolmeRuntimeCommitProviderError::MalformedResponse { reason }) => {
            assert!(
                reason.contains("invalid request")
                    || reason.contains("missing required field")
                    || reason.contains("txhash")
            );
        }
        Err(other) => {
            panic!("live node smoke expected endpoint reachability outcome, got error: {other:?}")
        }
    }
}

pub(crate) fn recover_pubkey_hex(
    message_json: &str,
    signature_hex: &str,
    recovery_id: u8,
) -> String {
    let signature_bytes =
        decode_hex_bytes(signature_hex).expect("signature hex must decode for recovery");
    let signature = Signature::from_slice(signature_bytes.as_slice())
        .expect("signature bytes must decode into a secp256k1 signature");
    let recovery = RecoveryId::from_byte(recovery_id).expect("recovery id must decode");
    let recovered = VerifyingKey::recover_from_msg(message_json.as_bytes(), &signature, recovery)
        .expect("signature must recover a verifying key");
    encode_hex_lower(recovered.to_encoded_point(true).as_bytes())
}

pub(crate) fn live_smoke_source() -> &'static str {
    include_str!("../live_smoke_contract_tests.rs")
}

pub(crate) fn live_smoke_fn_attributes(signature: &str) -> Vec<&'static str> {
    let lines: Vec<&'static str> = live_smoke_source().lines().collect();
    let fn_line = lines
        .iter()
        .position(|line| line.trim() == signature)
        .expect("live-node submit probe function must exist");
    collect_attributes(lines.as_slice(), fn_line)
}

fn collect_attributes(lines: &[&'static str], mut cursor: usize) -> Vec<&'static str> {
    let mut attributes = Vec::new();
    while cursor > 0 {
        cursor -= 1;
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("#[") {
            attributes.push(trimmed);
            continue;
        }
        break;
    }
    attributes
}
