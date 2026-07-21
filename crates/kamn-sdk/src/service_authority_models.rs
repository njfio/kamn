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
