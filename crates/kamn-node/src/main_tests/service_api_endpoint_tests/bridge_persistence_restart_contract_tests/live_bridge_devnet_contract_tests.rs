use super::super::*;
use super::support::{
    build_bridge_snapshot, forward_bridge, read_state_json, set_live_solana_bridge_rpc_url_env,
    set_state_file_env, submit_bridge, unique_named_state_file, write_live_bridge_proof_artifact,
};
use solana_commitment_config::CommitmentConfig;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_sdk::signer::keypair::{read_keypair_file, write_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use std::str::FromStr;
use std::time::Duration;

const DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const KEYPAIR_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const COMMITMENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT";
pub(super) const TRANSFER_LAMPORTS: u64 = 1_000_000;
const AIRDROP_LAMPORTS: u64 = 2_500_000;

struct LiveBridgeDevnetFixture {
    state_file: std::path::PathBuf,
    keypair_file: std::path::PathBuf,
    _state: EnvVarGuard,
    _rpc: EnvVarGuard,
    _keypair: EnvVarGuard,
    _recipient: EnvVarGuard,
    _lamports: EnvVarGuard,
    _commitment: EnvVarGuard,
    client: RpcClient,
    recipient: Keypair,
    recipient_before: u64,
}

#[test]
#[ignore = "requires live Solana devnet RPC access and airdrop availability"]
fn integration_live_bridge_receipt_matches_independent_finalized_rpc_evidence() {
    let _env = acquire_service_api_test_env();
    let fixture = LiveBridgeDevnetFixture::new();
    let snapshot = build_bridge_snapshot("127.0.0.1:34124");
    let bind = reserve_loopback_addr();
    let caller = "kamn:did:agent:live-bridge-devnet";
    let submitted = submit_bridge(
        &snapshot,
        &bind,
        caller,
        701,
        r#"{"payload":"devnet-proof"}"#,
    );
    let bridge_id = submitted["bridge_id"].as_str().expect("bridge id");
    let forwarded = forward_bridge(&snapshot, &bind, caller, 702, bridge_id);
    fixture.assert_authoritative_receipt(&forwarded);
    let restart = build_bridge_snapshot("127.0.0.1:34125");
    let repeated = forward_bridge(&restart, &reserve_loopback_addr(), caller, 703, bridge_id);
    assert_eq!(repeated["bridge_receipt"], forwarded["bridge_receipt"]);
    let recipient_after = fixture.assert_exactly_one_transfer();
    write_live_bridge_proof_artifact(&forwarded, fixture.recipient_before, recipient_after);
}

impl LiveBridgeDevnetFixture {
    fn new() -> Self {
        let client = rpc_client();
        let state_file = unique_named_state_file("kamn-live-bridge-devnet-state");
        let keypair_file = unique_named_state_file("kamn-live-bridge-devnet-keypair");
        let sender = funded_sender(&client);
        write_keypair_file(&sender, &keypair_file).expect("sender keypair should persist");
        let recipient = Keypair::new();
        let recipient_before = client
            .get_balance(&recipient.pubkey())
            .expect("recipient balance should resolve");
        let keypair_text = keypair_file.to_string_lossy().to_string();
        let recipient_text = recipient.pubkey().to_string();
        Self {
            _state: set_state_file_env(&state_file),
            _rpc: set_live_solana_bridge_rpc_url_env(Some(DEVNET_RPC_URL)),
            _keypair: EnvVarGuard::set(KEYPAIR_ENV, Some(&keypair_text)),
            _recipient: EnvVarGuard::set(RECIPIENT_ENV, Some(&recipient_text)),
            _lamports: EnvVarGuard::set(LAMPORTS_ENV, Some("1000000")),
            _commitment: EnvVarGuard::set(COMMITMENT_ENV, Some("finalized")),
            state_file,
            keypair_file,
            client,
            recipient,
            recipient_before,
        }
    }

    fn assert_authoritative_receipt(&self, forwarded: &Value) {
        let receipt = &forwarded["bridge_receipt"];
        let signature = Signature::from_str(
            receipt["transaction_signature"]
                .as_str()
                .expect("receipt transaction signature"),
        )
        .expect("receipt signature should decode");
        let response = self
            .client
            .get_signature_statuses_with_history(&[signature])
            .expect("independent RPC status query should succeed");
        let status = response.value[0]
            .as_ref()
            .expect("independent RPC status should exist");
        assert!(status.err.is_none());
        assert!(status.satisfies_commitment(CommitmentConfig::finalized()));
        assert_eq!(receipt["finalized_slot"].as_u64(), Some(status.slot));
        let bridge_id = forwarded["bridge_id"].as_str().expect("bridge id");
        let state = read_state_json(&self.state_file);
        assert_eq!(state["bridges"][bridge_id]["bridge_receipt"], *receipt);
    }

    fn assert_exactly_one_transfer(&self) -> u64 {
        let expected = self.recipient_before.saturating_add(TRANSFER_LAMPORTS);
        let actual = self
            .client
            .get_balance(&self.recipient.pubkey())
            .expect("recipient balance should resolve");
        assert_eq!(actual, expected);
        actual
    }
}

impl Drop for LiveBridgeDevnetFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.state_file);
        let _ = std::fs::remove_file(&self.keypair_file);
    }
}

fn rpc_client() -> RpcClient {
    RpcClient::new_with_timeout_and_commitment(
        DEVNET_RPC_URL.to_owned(),
        Duration::from_secs(30),
        CommitmentConfig::finalized(),
    )
}

fn funded_sender(client: &RpcClient) -> Keypair {
    if let Some(sender) = configured_funded_sender(client) {
        return sender;
    }
    let sender = Keypair::new();
    airdrop_and_wait(client, &sender);
    sender
}

fn configured_funded_sender(client: &RpcClient) -> Option<Keypair> {
    let home = std::env::var_os("HOME")?;
    let path = std::path::PathBuf::from(home).join(".config/solana/id.json");
    let sender = read_keypair_file(path).ok()?;
    let required = TRANSFER_LAMPORTS.saturating_add(10_000);
    (client.get_balance(&sender.pubkey()).ok()? >= required).then_some(sender)
}

fn airdrop_and_wait(client: &RpcClient, sender: &Keypair) {
    let signature = request_airdrop(client, &sender.pubkey());
    let confirmed = client
        .confirm_transaction_with_commitment(&signature, CommitmentConfig::finalized())
        .expect("airdrop confirmation should succeed");
    assert!(confirmed.value, "airdrop must finalize");
    wait_for_balance(client, &sender.pubkey(), AIRDROP_LAMPORTS);
}

fn request_airdrop(client: &RpcClient, pubkey: &solana_sdk::pubkey::Pubkey) -> Signature {
    let mut last_error = String::new();
    for _ in 0..12 {
        match client.request_airdrop(pubkey, AIRDROP_LAMPORTS) {
            Ok(signature) => return signature,
            Err(error) => {
                last_error = error.to_string();
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
    panic!("airdrop should succeed after retries: {last_error}");
}

fn wait_for_balance(client: &RpcClient, pubkey: &solana_sdk::pubkey::Pubkey, minimum: u64) {
    for _ in 0..30 {
        if client.get_balance(pubkey).unwrap_or_default() >= minimum {
            return;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    panic!("sender balance did not reach required minimum");
}
