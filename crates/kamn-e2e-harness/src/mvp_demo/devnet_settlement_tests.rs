use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::DevnetSettlementEvidence;

#[test]
fn unit_live_success_log_records_settlement_evidence() {
    let run_dir = temp_run_dir("devnet-success-log");
    std::fs::create_dir_all(run_dir.join("proof")).expect("proof dir should be created");

    super::write_live_success_log(&run_dir, &sample_evidence())
        .expect("live success log should be written");

    let log = std::fs::read_to_string(run_dir.join("proof/devnet-settlement-output.txt"))
        .expect("live success log should be readable");
    assert!(log.contains("devnet_settlement_status=PASS"));
    assert!(log.contains("settlement_tx_signature=devnet-signature-111"));
    assert!(log.contains("payer_balance_before=2500000000"));
    assert!(log.contains("recipient_balance_after=2501000000"));
    let _ = std::fs::remove_dir_all(&run_dir);
}

fn sample_evidence() -> DevnetSettlementEvidence {
    DevnetSettlementEvidence {
        network: "solana:devnet".to_owned(),
        execution_surface: "live-service-api".to_owned(),
        rpc_url: "https://api.devnet.solana.com".to_owned(),
        payer_pubkey: "payer111111111111111111111111111111111111111".to_owned(),
        recipient_pubkey: "recipient11111111111111111111111111111111111".to_owned(),
        lamports: 1_000_000,
        escrow_id: "escrow-local-test".to_owned(),
        task_id: None,
        task_binding_digest: None,
        settlement_tx_signature: "devnet-signature-111".to_owned(),
        settlement_commitment: "finalized".to_owned(),
        payer_balance_before: 2_500_000_000,
        payer_balance_after: 2_498_995_000,
        recipient_balance_before: 2_500_000_000,
        recipient_balance_after: 2_501_000_000,
        persisted_settlement_tx_signature: "devnet-signature-111".to_owned(),
        authoritative_rpc_artifact: None,
    }
}

fn temp_run_dir(prefix: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-{prefix}-{}-{suffix}", std::process::id()))
}
