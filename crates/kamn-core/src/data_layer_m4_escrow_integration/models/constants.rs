/// Hash algorithm label used by M4 deterministic digests.
pub const DATA_LAYER_M4_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker used by per-escrow settlement evidence hash chains.
pub const DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS: &str = "GENESIS";
/// Transition reason marker for `Created -> Funded`.
pub const DATA_LAYER_M4_ESCROW_FUNDED_REASON_CODE: &str = "m4_escrow_funded";
/// Transition reason marker for `Funded -> Active`.
pub const DATA_LAYER_M4_ESCROW_ACTIVE_REASON_CODE: &str = "m4_escrow_active";
/// Transition reason marker for `Active -> Disputed`.
pub const DATA_LAYER_M4_ESCROW_DISPUTED_REASON_CODE: &str = "m4_escrow_disputed";
/// Transition reason marker for release settlement.
pub const DATA_LAYER_M4_ESCROW_RELEASED_REASON_CODE: &str = "m4_escrow_released";
/// Transition reason marker for refund settlement.
pub const DATA_LAYER_M4_ESCROW_REFUNDED_REASON_CODE: &str = "m4_escrow_refunded";
/// Transition reason marker for expiry settlement.
pub const DATA_LAYER_M4_ESCROW_EXPIRED_REASON_CODE: &str = "m4_escrow_expired";
/// Visibility reason marker for participant scope allow.
pub const DATA_LAYER_M4_ESCROW_PARTICIPANT_SCOPE_ALLOWED_REASON_CODE: &str =
    "m4_escrow_participant_scope_allowed";
/// Visibility reason marker when auditor tries access outside disputed state.
pub const DATA_LAYER_M4_ESCROW_AUDITOR_DISPUTE_REQUIRED_REASON_CODE: &str =
    "m4_escrow_auditor_dispute_required";
/// Visibility reason marker when auditor threshold is missing.
pub const DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_CONFIGURED_REASON_CODE: &str =
    "m4_escrow_auditor_threshold_not_configured";
/// Visibility reason marker when auditor threshold is satisfied.
pub const DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE: &str =
    "m4_escrow_auditor_scope_allowed";
/// Visibility reason marker when auditor threshold is not met.
pub const DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_MET_REASON_CODE: &str =
    "m4_escrow_auditor_threshold_not_met";
/// Visibility reason marker for denied non-participant scope.
pub const DATA_LAYER_M4_ESCROW_SCOPE_DENIED_REASON_CODE: &str = "m4_escrow_scope_denied";
/// Reconciliation reason marker for escrow/evidence match.
pub const DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MATCH_REASON_CODE: &str =
    "m4_settlement_evidence_reconciliation_match";
/// Reconciliation reason marker for escrow/evidence mismatch.
pub const DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MISMATCH_REASON_CODE: &str =
    "m4_settlement_evidence_reconciliation_mismatch";
