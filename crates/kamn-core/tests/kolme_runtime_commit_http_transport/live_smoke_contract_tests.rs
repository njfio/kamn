use super::*;

#[path = "live_smoke_contract_tests/support.rs"]
mod support;

use support::*;

#[test]
fn integration_kolme_fork_live_node_submit_reaches_endpoint() {
    if !local_heavy_enabled() {
        return;
    }

    let base_url = required_live_env("KAMN_KOLME_LIVE_BASE_URL");
    let provider_hint = optional_live_env("KAMN_KOLME_LIVE_PROVIDER_HINT", "kolme-fork-local");
    assert_live_signing_profile();

    let (wire_payload, idempotency_key) = live_smoke_payload();
    let outcome = live_smoke_provider(base_url.as_str(), provider_hint.as_str())
        .submit_runtime_commit(wire_payload.as_str(), idempotency_key.as_str());
    assert_live_reachability_outcome(outcome);
}

#[test]
fn unit_kolme_fork_live_smoke_signer_emits_recoverable_secp256k1_signature() {
    let pubkey = kolme_fork_live_smoke_pubkey_hex();
    let message_json = live_smoke_message_json(pubkey.as_str(), 7);
    let (signature_hex, recovery_id) = kolme_fork_sign_message(message_json.as_str());

    assert_eq!(signature_hex.len(), 128);
    assert!(signature_hex.chars().all(|character| character.is_ascii_hexdigit()));
    assert!(recovery_id <= 3, "recovery id must be in secp256k1 range");
    assert_eq!(recover_pubkey_hex(message_json.as_str(), signature_hex.as_str(), recovery_id), pubkey);
}

#[test]
fn regression_live_node_smoke_signature_payload_must_not_use_synthetic_literal() {
    assert!(!live_smoke_source().contains("\\\"signature\\\":\\\"sig-live-smoke-"));
}

#[test]
fn regression_live_node_submit_probe_must_not_be_ignored() {
    assert!(
        live_smoke_fn_attributes("fn integration_kolme_fork_live_node_submit_reaches_endpoint() {")
            .iter()
            .all(|line| !line.contains("ignore")),
        "live-node submit probe must stay active; local-heavy gating belongs in runtime preflight, not #[ignore]"
    );
}
