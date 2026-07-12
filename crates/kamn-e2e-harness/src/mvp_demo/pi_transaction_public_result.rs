use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PublicResult {
    pub(super) did: Option<String>,
    pub(super) task_id: Option<String>,
    pub(super) state: Option<String>,
    pub(super) transaction_id: Option<String>,
    pub(super) escrow_id: Option<String>,
    pub(super) amount_lamports: Option<u64>,
    pub(super) network: Option<String>,
    pub(super) settlement_tx_signature: Option<String>,
    pub(super) settlement_commitment: Option<String>,
    pub(super) public_commitment: Option<String>,
    pub(super) view_scope: Option<String>,
    pub(super) participant_role: Option<String>,
}

pub(super) fn validate_public_result(result: &PublicResult, is_error: bool) -> Result<(), String> {
    let values = result.string_values();
    if values.iter().flatten().any(|value| value.is_empty()) || result.amount_lamports == Some(0) {
        return Err(invalid());
    }
    let is_empty = values.iter().all(|value| value.is_none()) && result.amount_lamports.is_none();
    if is_error != is_empty {
        return Err(invalid());
    }
    Ok(())
}

impl PublicResult {
    fn string_values(&self) -> [Option<&str>; 11] {
        [
            self.did.as_deref(),
            self.task_id.as_deref(),
            self.state.as_deref(),
            self.transaction_id.as_deref(),
            self.escrow_id.as_deref(),
            self.network.as_deref(),
            self.settlement_tx_signature.as_deref(),
            self.settlement_commitment.as_deref(),
            self.public_commitment.as_deref(),
            self.view_scope.as_deref(),
            self.participant_role.as_deref(),
        ]
    }
}

fn invalid() -> String {
    "RUNTIME_RECEIPT_CHAIN_PUBLIC_RESULT_INVALID".to_owned()
}
