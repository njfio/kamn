#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveSettlementEvidence {
    pub(crate) settlement_receipt_hash: String,
    pub(crate) settlement_tx_signature: String,
    pub(crate) settlement_network: String,
    pub(crate) settlement_commitment: String,
}
