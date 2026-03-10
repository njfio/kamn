use super::*;
#[test]
fn integration_kolme_fork_live_node_submit_reaches_endpoint() {
    if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping live-node smoke; set KAMN_KOLME_LOCAL_HEAVY=1 to run local-heavy live probe"
        );
        return;
    }

    let base_url = env::var("KAMN_KOLME_LIVE_BASE_URL")
        .expect("KAMN_KOLME_LIVE_BASE_URL must be set for live node smoke");
    let provider_hint =
        env::var("KAMN_KOLME_LIVE_PROVIDER_HINT").unwrap_or_else(|_| "kolme-fork-local".to_owned());
    let signing_profile = env::var(KOLME_LIVE_SIGNING_PROFILE_ENV)
        .unwrap_or_else(|_| KOLME_FORK_SECP256K1_PROFILE.to_owned());
    let authorization_header = env::var("KAMN_KOLME_LIVE_AUTHORIZATION").ok();
    assert_eq!(
        signing_profile, KOLME_FORK_SECP256K1_PROFILE,
        "live node smoke must use fork-compatible secp256k1 signing profile"
    );

    let transport = if let Some(value) = authorization_header {
        KolmeRuntimeCommitHttpTransport::new_with_authorization(10, value.as_str())
            .expect("transport with authorization should build")
    } else {
        KolmeRuntimeCommitHttpTransport::new(10).expect("transport should build")
    };

    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        provider_hint.as_str(),
        transport,
    )
    .expect("provider should build");

    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let signer_pubkey = kolme_fork_live_smoke_pubkey_hex();
    let nonce = ((unique_suffix % 1_000_000_000_u128) as u64) + 1;
    let message_json = format!(
        "{{\"pubkey\":\"{signer_pubkey}\",\"nonce\":{nonce},\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}}"
    );
    let (signature, recovery_id) = kolme_fork_sign_message(message_json.as_str());
    let wire_payload = format!(
        "{{\"message\":\"{}\",\"signature\":\"{}\",\"recovery_id\":{}}}",
        message_json.replace('\\', "\\\\").replace('\"', "\\\""),
        signature,
        recovery_id
    );
    let idempotency_key = format!("kolme-runtime-commit:live-smoke:{unique_suffix}");

    let outcome = provider.submit_runtime_commit(wire_payload.as_str(), idempotency_key.as_str());
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
                    || reason.contains("txhash"),
                "unexpected malformed response reason from live node: {reason}"
            );
        }
        Err(other) => {
            panic!("live node smoke expected endpoint reachability outcome, got error: {other:?}");
        }
    }
}

#[test]
fn unit_kolme_fork_live_smoke_signer_emits_recoverable_secp256k1_signature() {
    let pubkey = kolme_fork_live_smoke_pubkey_hex();
    let message_json = format!(
        "{{\"pubkey\":\"{pubkey}\",\"nonce\":7,\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}}"
    );
    let (signature_hex, recovery_id) = kolme_fork_sign_message(message_json.as_str());

    assert_eq!(signature_hex.len(), 128);
    assert!(
        signature_hex
            .chars()
            .all(|character| character.is_ascii_hexdigit()),
        "signature must be lowercase/uppercase hex bytes"
    );
    assert!(recovery_id <= 3, "recovery id must be in secp256k1 range");

    let signature_bytes =
        decode_hex_bytes(signature_hex.as_str()).expect("signature hex must decode for recovery");
    let signature = Signature::from_slice(signature_bytes.as_slice())
        .expect("signature bytes must decode into a secp256k1 signature");
    let recovery = RecoveryId::from_byte(recovery_id).expect("recovery id must decode");
    let recovered_key =
        VerifyingKey::recover_from_msg(message_json.as_bytes(), &signature, recovery)
            .expect("signature must recover a verifying key");
    let recovered_pubkey = encode_hex_lower(recovered_key.to_encoded_point(true).as_bytes());
    assert_eq!(recovered_pubkey, pubkey);
}

#[test]
fn regression_live_node_smoke_signature_payload_must_not_use_synthetic_literal() {
    const SOURCE: &str = include_str!("../kolme_runtime_commit_http_transport/live_smoke_contract_tests.rs");
    assert!(
        !SOURCE.contains("\\\"signature\\\":\\\"sig-live-smoke-"),
        "live-node smoke payload must use real secp256k1 signature generation"
    );
}

#[test]
fn regression_live_node_submit_probe_must_not_be_ignored() {
    const SOURCE: &str = include_str!("../kolme_runtime_commit_http_transport/live_smoke_contract_tests.rs");
    let lines: Vec<&str> = SOURCE.lines().collect();
    let fn_line = lines
        .iter()
        .position(|line| {
            line.trim() == "fn integration_kolme_fork_live_node_submit_reaches_endpoint() {"
        })
        .expect("live-node submit probe function must exist");
    let mut attributes = Vec::new();
    let mut cursor = fn_line;
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

    assert!(
        attributes.iter().all(|line| !line.contains("ignore")),
        "live-node submit probe must stay active; local-heavy gating belongs in runtime preflight, not #[ignore]"
    );
}
