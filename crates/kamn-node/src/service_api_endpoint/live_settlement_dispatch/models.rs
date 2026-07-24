#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveSettlementEvidence {
    pub(crate) settlement_receipt_hash: String,
    pub(crate) settlement_tx_signature: String,
    pub(crate) settlement_network: String,
    pub(crate) settlement_commitment: String,
    pub(crate) recipient_pubkey: Option<String>,
    pub(crate) amount_lamports: Option<u64>,
    pub(crate) finalized_slot: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedLiveSettlement {
    pub(crate) expected_signature: String,
    pub(crate) signed_transaction_digest: String,
    pub(crate) signed_transaction_json: String,
    pub(crate) recipient_pubkey: String,
    pub(crate) amount_lamports: u64,
    pub(crate) network: String,
}

pub(super) fn build_live_settlement_evidence(
    signature: String,
    commitment: &str,
    prepared: &PreparedLiveSettlement,
) -> LiveSettlementEvidence {
    LiveSettlementEvidence {
        settlement_receipt_hash: signature.clone(),
        settlement_tx_signature: signature,
        settlement_network: "solana:devnet".to_owned(),
        settlement_commitment: commitment.to_owned(),
        recipient_pubkey: Some(prepared.recipient_pubkey.clone()),
        amount_lamports: Some(prepared.amount_lamports),
        finalized_slot: None,
    }
}
