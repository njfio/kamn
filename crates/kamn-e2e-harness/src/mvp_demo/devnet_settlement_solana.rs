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
    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    stdout
        .split_whitespace()
        .next()
        .ok_or_else(|| "solana balance output was empty".to_owned())?
        .parse::<u64>()
        .map_err(|error| format!("solana balance output was not lamports: {error}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_parse_solana_lamports_accepts_cli_suffix() {
        let output = std::process::Output {
            status: success_status(),
            stdout: b"2498995000 lamports\n".to_vec(),
            stderr: Vec::new(),
        };
        let parsed = super::parse_solana_lamports_output(&output)
            .expect("lamport parser should accept solana CLI suffix");
        assert_eq!(parsed, 2_498_995_000);
    }

    #[cfg(unix)]
    fn success_status() -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
}
