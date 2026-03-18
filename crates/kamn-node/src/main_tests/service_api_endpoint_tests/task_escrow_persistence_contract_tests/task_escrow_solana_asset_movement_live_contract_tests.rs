use super::super::*;
use super::support::{
    build_task_escrow_snapshot, fund_escrow, release_escrow, set_live_solana_bridge_rpc_url_env,
    set_state_file_env, unique_named_state_file,
};
use solana_commitment_config::CommitmentConfig;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::{write_keypair_file, Keypair};
use solana_sdk::signer::Signer;
use std::time::Duration;

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const SOLANA_SETTLEMENT_RECIPIENT_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const SOLANA_SETTLEMENT_LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const LIVE_SETTLEMENT_LAMPORTS: u64 = 1_000_000;
const LIVE_AIRDROP_LAMPORTS: u64 = 2_500_000;

struct LiveDevnetAssetMovementFixture {
    state_file: std::path::PathBuf,
    sender_keypair_file: std::path::PathBuf,
    _state_file_guard: EnvVarGuard,
    _live_rpc_guard: EnvVarGuard,
    _keypair_guard: EnvVarGuard,
    _recipient_guard: EnvVarGuard,
    _lamports_guard: EnvVarGuard,
    client: RpcClient,
    recipient: Keypair,
    recipient_before: u64,
}

#[test]
#[ignore = "requires live Solana devnet RPC access and airdrop availability"]
fn integration_service_api_endpoint_live_solana_asset_movement_release_submits_real_devnet_transfer(
) {
    let _env = acquire_service_api_test_env();
    let fixture = build_live_devnet_asset_movement_fixture();
    let released = submit_live_asset_movement_release();
    assert_live_release_result(&fixture, &released);
}

impl Drop for LiveDevnetAssetMovementFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.state_file.as_path());
        let _ = std::fs::remove_file(self.sender_keypair_file.as_path());
    }
}

fn airdrop_and_wait(client: &RpcClient, pubkey: &Pubkey, lamports: u64) {
    let signature = request_airdrop_with_retry(client, pubkey, lamports);
    let confirmed = client
        .confirm_transaction_with_commitment(&signature, CommitmentConfig::finalized())
        .expect("airdrop confirmation should succeed");
    assert!(confirmed.value, "airdrop must finalize before release");
    wait_for_balance_at_least(client, pubkey, lamports);
}

fn request_airdrop_with_retry(client: &RpcClient, pubkey: &Pubkey, lamports: u64) -> solana_sdk::signature::Signature {
    let mut last_error = String::new();
    for _ in 0..12 {
        match client.request_airdrop(pubkey, lamports) {
            Ok(signature) => return signature,
            Err(error) => {
                last_error = error.to_string();
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
    panic!("airdrop request should succeed after retries: {last_error}");
}

fn build_live_devnet_asset_movement_fixture() -> LiveDevnetAssetMovementFixture {
    let client = devnet_rpc_client();
    let state_file = unique_named_state_file("kamn-node-solana-asset-movement-live-state");
    let (_state_file_text, state_file_guard) = set_state_file_env(state_file.as_path());
    let sender_keypair_file = unique_named_state_file("kamn-node-solana-asset-movement-live-keypair");
    let sender = Keypair::new();
    write_keypair_file(&sender, sender_keypair_file.as_path())
        .expect("sender keypair file should write");
    let recipient = Keypair::new();
    let guards = configure_live_asset_movement_env(&sender_keypair_file, &recipient);
    let recipient_before = balance(&client, &recipient.pubkey());
    airdrop_and_wait(&client, &sender.pubkey(), LIVE_AIRDROP_LAMPORTS);
    LiveDevnetAssetMovementFixture {
        state_file,
        sender_keypair_file,
        _state_file_guard: state_file_guard,
        _live_rpc_guard: guards.0,
        _keypair_guard: guards.1,
        _recipient_guard: guards.2,
        _lamports_guard: guards.3,
        client,
        recipient,
        recipient_before,
    }
}

fn devnet_rpc_client() -> RpcClient {
    RpcClient::new_with_timeout_and_commitment(
        LIVE_SOLANA_DEVNET_RPC_URL.to_owned(),
        Duration::from_secs(30),
        CommitmentConfig::finalized(),
    )
}

fn configure_live_asset_movement_env(
    sender_keypair_file: &std::path::Path,
    recipient: &Keypair,
) -> (EnvVarGuard, EnvVarGuard, EnvVarGuard, EnvVarGuard) {
    let sender_keypair_file_text = sender_keypair_file.to_string_lossy().to_string();
    let recipient_pubkey_text = recipient.pubkey().to_string();
    let lamports_text = LIVE_SETTLEMENT_LAMPORTS.to_string();
    (
        set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL)),
        EnvVarGuard::set(
            SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
            Some(sender_keypair_file_text.as_str()),
        ),
        EnvVarGuard::set(
            SOLANA_SETTLEMENT_RECIPIENT_ENV,
            Some(recipient_pubkey_text.as_str()),
        ),
        EnvVarGuard::set(
            SOLANA_SETTLEMENT_LAMPORTS_ENV,
            Some(lamports_text.as_str()),
        ),
    )
}

fn submit_live_asset_movement_release() -> Value {
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34132");
    let bind_addr = reserve_loopback_addr();
    let funded = fund_escrow(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:test-client-solana-asset-movement-live",
        201,
        r#"{"task_id":"solana-asset-movement-live-task","amount":9}"#,
    );
    let escrow_id = funded["escrow_id"]
        .as_str()
        .expect("escrow id should be string");
    release_escrow(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:test-client-solana-asset-movement-live",
        202,
        escrow_id,
    )
}

fn assert_live_release_result(
    fixture: &LiveDevnetAssetMovementFixture,
    released: &Value,
) {
    let signature = released["settlement_tx_signature"]
        .as_str()
        .expect("release must expose a real Solana transaction signature");
    wait_for_balance_at_least(
        &fixture.client,
        &fixture.recipient.pubkey(),
        fixture
            .recipient_before
            .saturating_add(LIVE_SETTLEMENT_LAMPORTS),
    );
    assert_eq!(released["settlement_network"], "solana:devnet");
    assert_eq!(released["settlement_commitment"], "finalized");
    assert_eq!(released["settlement_receipt_hash"], signature);
}

fn wait_for_balance_at_least(client: &RpcClient, pubkey: &Pubkey, minimum: u64) {
    for _ in 0..30 {
        if balance(client, pubkey) >= minimum {
            return;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    panic!("expected balance of at least {minimum} lamports for {pubkey}");
}

fn balance(client: &RpcClient, pubkey: &Pubkey) -> u64 {
    client
        .get_balance(pubkey)
        .expect("balance lookup should succeed")
}
