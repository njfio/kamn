use serde_json::Value;
use std::path::Path;

use super::settlement_evidence_artifact::SettlementEvidenceArtifact;

pub(super) fn validate_settlement_log(
    report: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    let path = report["artifacts"]["devnet_settlement_output"]
        .as_str()
        .ok_or_else(invalid)?;
    let log = std::fs::read_to_string(Path::new(path)).map_err(|_| invalid())?;
    require(&log, "devnet_settlement_status", "PASS")?;
    require(&log, "network", evidence.network.as_str())?;
    require(&log, "rpc_url", evidence.rpc_url.as_str())?;
    require(&log, "payer_pubkey", evidence.payer_pubkey.as_str())?;
    require(&log, "recipient_pubkey", evidence.recipient_pubkey.as_str())?;
    require(&log, "lamports", evidence.lamports.to_string().as_str())?;
    require(&log, "escrow_id", evidence.escrow_id.as_str())?;
    require(
        &log,
        "settlement_tx_signature",
        evidence.settlement_tx_signature.as_str(),
    )?;
    require(
        &log,
        "settlement_commitment",
        evidence.settlement_commitment.as_str(),
    )?;
    require_balances(&log, evidence)
}

fn require_balances(log: &str, evidence: &SettlementEvidenceArtifact) -> Result<(), String> {
    for (field, value) in [
        ("payer_balance_before", evidence.payer_balance_before),
        ("payer_balance_after", evidence.payer_balance_after),
        (
            "recipient_balance_before",
            evidence.recipient_balance_before,
        ),
        ("recipient_balance_after", evidence.recipient_balance_after),
    ] {
        require(log, field, value.to_string().as_str())?;
    }
    require(
        log,
        "persisted_settlement_tx_signature",
        evidence.persisted_settlement_tx_signature.as_str(),
    )
}

fn require(log: &str, field: &str, expected: &str) -> Result<(), String> {
    let marker = format!("{field}={expected}");
    if log.lines().any(|line| line == marker) {
        return Ok(());
    }
    Err(invalid())
}

fn invalid() -> String {
    "SETTLEMENT_EVIDENCE_INVALID".to_owned()
}
