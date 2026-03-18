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

#[test]
#[ignore = "requires live Solana devnet RPC access and airdrop availability"]
fn integration_service_api_endpoint_live_solana_asset_movement_release_submits_real_devnet_transfer(
) {
    let _env = acquire_service_api_test_env();
    let client = RpcClient::new_with_timeout_and_commitment(
        LIVE_SOLANA_DEVNET_RPC_URL.to_owned(),
        Duration::from_secs(30),
        CommitmentConfig::finalized(),
    );
    let state_file = unique_named_state_file("kamn-node-solana-asset-movement-live-state");
    let (_state_file_text, _state_file_guard) = set_state_file_env(state_file.as_path());
    let sender_keypair_file = unique_named_state_file("kamn-node-solana-asset-movement-live-keypair");
    let sender = Keypair::new();
    write_keypair_file(&sender, sender_keypair_file.as_path())
        .expect("sender keypair file should write");
    let recipient = Keypair::new();
    let sender_keypair_file_text = sender_keypair_file.to_string_lossy().to_string();
    let recipient_pubkey_text = recipient.pubkey().to_string();
    let lamports_text = LIVE_SETTLEMENT_LAMPORTS.to_string();
    let recipient_before = balance(&client, &recipient.pubkey());

    let _live_rpc_guard = set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let _keypair_guard = EnvVarGuard::set(
        SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        Some(sender_keypair_file_text.as_str()),
    );
    let _recipient_guard = EnvVarGuard::set(
        SOLANA_SETTLEMENT_RECIPIENT_ENV,
        Some(recipient_pubkey_text.as_str()),
    );
    let _lamports_guard = EnvVarGuard::set(
        SOLANA_SETTLEMENT_LAMPORTS_ENV,
        Some(lamports_text.as_str()),
    );
    airdrop_and_wait(&client, &sender.pubkey(), LIVE_AIRDROP_LAMPORTS);

    let snapshot = build_task_escrow_snapshot("127.0.0.1:34132");
    let bind_addr = reserve_loopback_addr();
    let caller_did = "kamn:did:agent:test-client-solana-asset-movement-live";
    let funded = fund_escrow(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        201,
        r#"{"task_id":"solana-asset-movement-live-task","amount":9}"#,
    );
    let escrow_id = funded["escrow_id"]
        .as_str()
        .expect("escrow id should be string");
    let released = release_escrow(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        202,
        escrow_id,
    );
    let signature = released["settlement_tx_signature"]
        .as_str()
        .expect("release must expose a real Solana transaction signature");

    wait_for_balance_at_least(
        &client,
        &recipient.pubkey(),
        recipient_before.saturating_add(LIVE_SETTLEMENT_LAMPORTS),
    );
    assert_eq!(released["settlement_network"], "solana:devnet");
    assert_eq!(released["settlement_commitment"], "finalized");
    assert_eq!(released["settlement_receipt_hash"], signature);

    let _ = std::fs::remove_file(state_file);
    let _ = std::fs::remove_file(sender_keypair_file);
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
