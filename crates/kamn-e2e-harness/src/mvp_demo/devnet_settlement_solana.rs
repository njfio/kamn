use std::process::Command;

use super::devnet_settlement_live::LiveSettlementConfig;

pub(super) struct BalancePair {
    pub(super) payer: u64,
    pub(super) recipient: u64,
}

pub(super) fn balance_pair(
    rpc_url: &str,
    payer: &str,
    config: &LiveSettlementConfig,
) -> Result<BalancePair, String> {
    Ok(BalancePair {
        payer: solana_balance_lamports(rpc_url, payer, config.commitment.as_str())?,
        recipient: solana_balance_lamports(
            rpc_url,
            config.recipient_pubkey.as_str(),
            config.commitment.as_str(),
        )?,
    })
}

pub(super) fn solana_pubkey(keypair_file: &str) -> Result<String, String> {
    let output = Command::new("solana-keygen")
        .args(["pubkey", keypair_file])
        .output()
        .map_err(|error| format!("failed to run solana-keygen pubkey: {error}"))?;
    if !output.status.success() {
        return Err("solana-keygen pubkey failed".to_owned());
    }
    Ok(String::from_utf8_lossy(output.stdout.as_slice())
        .trim()
        .to_owned())
}

pub(super) fn validate_balance_movement(
    before: &BalancePair,
    after: &BalancePair,
    config: &LiveSettlementConfig,
) -> Result<(), String> {
    if after.recipient < before.recipient.saturating_add(config.lamports) {
        return Err("recipient balance did not increase by configured lamports".to_owned());
    }
    if after.payer >= before.payer {
        return Err("payer balance did not decrease after settlement transfer".to_owned());
    }
    Ok(())
}

fn solana_balance_lamports(rpc_url: &str, pubkey: &str, commitment: &str) -> Result<u64, String> {
    let output = Command::new("solana")
        .args([
            "balance",
            "--lamports",
            "--url",
            rpc_url,
            "--commitment",
            commitment,
            pubkey,
        ])
        .output()
        .map_err(|error| format!("failed to run solana balance: {error}"))?;
    parse_solana_lamports_output(&output)
}

fn parse_solana_lamports_output(output: &std::process::Output) -> Result<u64, String> {
    if !output.status.success() {
        return Err(format!(
            "solana balance failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice())
        ));
    }
    String::from_utf8_lossy(output.stdout.as_slice())
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("solana balance output was not lamports: {error}"))
}
