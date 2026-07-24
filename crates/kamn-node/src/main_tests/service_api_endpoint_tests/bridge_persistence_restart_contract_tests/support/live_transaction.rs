use super::*;
use solana_sdk::signer::keypair::{write_keypair_file, Keypair};
use solana_sdk::signer::Signer;

const KEYPAIR_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const COMMITMENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT";

pub(crate) struct LiveBridgeTransactionEnv {
    keypair_file: PathBuf,
    _rpc: EnvVarGuard,
    _keypair: EnvVarGuard,
    _recipient: EnvVarGuard,
    _lamports: EnvVarGuard,
    _commitment: EnvVarGuard,
}

impl LiveBridgeTransactionEnv {
    pub(crate) fn enable(label: &str) -> Self {
        let keypair = Keypair::new();
        let recipient = Keypair::new().pubkey().to_string();
        let keypair_file = unique_named_state_file(label);
        write_keypair_file(&keypair, &keypair_file).expect("test keypair should persist");
        let keypair_text = keypair_file.to_string_lossy().to_string();
        Self {
            keypair_file,
            _rpc: set_default_live_solana_bridge_rpc_url_env(),
            _keypair: EnvVarGuard::set(KEYPAIR_ENV, Some(keypair_text.as_str())),
            _recipient: EnvVarGuard::set(RECIPIENT_ENV, Some(recipient.as_str())),
            _lamports: EnvVarGuard::set(LAMPORTS_ENV, Some("1")),
            _commitment: EnvVarGuard::set(COMMITMENT_ENV, Some("finalized")),
        }
    }
}

impl Drop for LiveBridgeTransactionEnv {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.keypair_file);
    }
}

pub(crate) fn assert_non_placeholder_bridge_evidence(bridge_id: &str, forwarded: &Value) {
    assert_ne!(
        forwarded["target_message_id"],
        Value::String(format!("msg-bridge-target-{bridge_id}"))
    );
    assert_ne!(
        forwarded["forward_tx_hash"],
        Value::String(format!("sha256:bridge-forwarded-{bridge_id}"))
    );
}

pub(crate) fn assert_non_placeholder_bridge_payload(bridge_id: &str, queried: &Value) {
    assert_non_placeholder_bridge_evidence(bridge_id, queried);
}

pub(crate) fn submit_and_forward_live_bridge(
    caller_did: &str,
    snapshot_addr: &str,
    submit_request_id: u64,
    forward_request_id: u64,
    request_body: &str,
) -> (String, Value) {
    let snapshot = build_bridge_snapshot(snapshot_addr);
    let bind_addr = reserve_loopback_addr();
    let submitted = submit_bridge(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        submit_request_id,
        request_body,
    );
    let bridge_id = submitted["bridge_id"]
        .as_str()
        .expect("bridge id should be string")
        .to_owned();
    let forwarded = forward_bridge(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        forward_request_id,
        bridge_id.as_str(),
    );
    (bridge_id, forwarded)
}

pub(crate) fn submit_and_restart_live_bridge(
    caller_did: &str,
    snapshot_addrs: (&str, &str),
    request_ids: (u64, u64, u64),
    request_body: &str,
) -> (String, Value) {
    let (bridge_id, _) = submit_and_forward_live_bridge(
        caller_did,
        snapshot_addrs.0,
        request_ids.0,
        request_ids.1,
        request_body,
    );
    let restart_snapshot = build_bridge_snapshot(snapshot_addrs.1);
    let queried = query_bridge(
        &restart_snapshot,
        reserve_loopback_addr().as_str(),
        caller_did,
        request_ids.2,
        bridge_id.as_str(),
    );
    (bridge_id, queried)
}
