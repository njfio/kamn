/// Parsed response for task lifecycle mutations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTaskTransitionReceipt {
    /// Task identifier.
    pub task_id: String,
    /// Resulting lifecycle state.
    pub state: String,
    /// Service-issued durable receipt identifier.
    pub receipt_id: String,
    /// Digest of the durable service receipt.
    pub receipt_digest: String,
    /// Canonical durable receipt action.
    pub action: String,
}

/// Durable settlement receipt returned with a finalized escrow release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSettlementReceipt {
    /// Settlement intent receipt identifier.
    pub receipt_id: String,
    /// Digest of the durable settlement intent.
    pub receipt_digest: String,
    /// Canonical settlement receipt action.
    pub action: String,
    /// Escrow identifier bound by the settlement receipt.
    pub resource_id: String,
    /// Durable settlement intent state.
    pub state: String,
}

/// Complete service-issued authority for a bridge-backed settlement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAuthoritativeSettlement {
    /// Bridge operation identifier.
    pub bridge_id: String,
    /// Finalized bridge receipt identifier.
    pub bridge_receipt_id: String,
    /// Digest of the finalized bridge receipt.
    pub bridge_receipt_digest: String,
    /// Settlement intent receipt identifier.
    pub settlement_receipt_id: String,
    /// Digest of the settlement intent receipt.
    pub settlement_receipt_digest: String,
    /// Canonical settlement action.
    pub action: String,
    /// Escrow resource bound by the settlement.
    pub resource_id: String,
    /// Service-authenticated settlement actor.
    pub actor_did: String,
    /// Durable settlement result.
    pub resulting_state: String,
    /// Bound task identifier.
    pub task_id: String,
    /// Bound escrow identifier.
    pub escrow_id: String,
    /// Settlement recipient.
    pub recipient: String,
    /// Settled amount in lamports.
    pub amount_lamports: u64,
    /// Settled asset marker.
    pub asset: String,
    /// Settlement network.
    pub network: String,
    /// Finalized transaction signature.
    pub transaction_signature: String,
    /// Finality commitment.
    pub commitment: String,
    /// Finalized network slot.
    pub finalized_slot: u64,
    /// Commitment to the full receipt chain.
    pub receipt_chain_commitment: String,
    /// Commitment to agreed economic terms.
    pub terms_digest: String,
    /// Durable operation identity.
    pub idempotency_key: String,
}

/// Finalized bridge receipt authority returned by bridge status routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceBridgeReceipt {
    /// Receipt identifier.
    pub receipt_id: String,
    /// Receipt digest.
    pub receipt_digest: String,
    /// Bridge identifier.
    pub bridge_id: String,
    /// Finalized transaction signature.
    pub transaction_signature: String,
    /// Settlement network.
    pub network: String,
    /// Finality commitment.
    pub commitment: String,
    /// Finalized network slot.
    pub finalized_slot: u64,
    /// Canonical bridge action.
    pub action: String,
    /// Bound bridge resource.
    pub resource_id: String,
    /// Durable bridge state.
    pub state: String,
}
