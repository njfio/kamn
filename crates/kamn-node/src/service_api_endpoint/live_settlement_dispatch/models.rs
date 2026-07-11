#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveSettlementEvidence {
    pub(crate) settlement_receipt_hash: String,
    pub(crate) settlement_tx_signature: String,
    pub(crate) settlement_network: String,
    pub(crate) settlement_commitment: String,
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
