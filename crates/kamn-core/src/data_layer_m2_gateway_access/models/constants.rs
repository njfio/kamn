/// PostgreSQL session variable key used by RLS predicates.
pub const DATA_LAYER_M2_REQUESTER_DID_SETTING: &str = "kamn.requester_did";
/// Hash algorithm label used by M2 deterministic digests.
pub const DATA_LAYER_M2_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker for access-audit hash chains.
pub const DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS: &str = "GENESIS";
/// Reason marker for agent sender/recipient scope access.
pub const DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED: &str =
    "m2_agent_counterparty_scope_allowed";
/// Reason marker for owner supervisory scope access.
pub const DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED: &str = "m2_owner_scope_allowed";
/// Reason marker for dispute-scoped escrow auditor access.
pub const DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED: &str =
    "m2_escrow_auditor_scope_allowed";
/// Reason marker for fail-closed ABAC denials.
pub const DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED: &str = "m2_abac_scope_denied";
/// Negative authorization matrix result marker when all cases deny as expected.
pub const DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE: &str =
    "m2_negative_matrix_all_denied";
/// Negative authorization matrix result marker when any case drifts.
pub const DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE: &str =
    "m2_negative_matrix_drift_detected";
/// Invalid requester DID reason marker.
pub const DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE: &str = "m2_invalid_requester_did";
/// Invalid sender DID reason marker.
pub const DATA_LAYER_M2_INVALID_SENDER_DID_REASON_CODE: &str = "m2_invalid_sender_did";
/// Invalid recipient DID reason marker.
pub const DATA_LAYER_M2_INVALID_RECIPIENT_DID_REASON_CODE: &str = "m2_invalid_recipient_did";
/// Invalid owner sender DID reason marker.
pub const DATA_LAYER_M2_INVALID_OWNER_SENDER_DID_REASON_CODE: &str = "m2_invalid_owner_sender_did";
/// Invalid owner recipient DID reason marker.
pub const DATA_LAYER_M2_INVALID_OWNER_RECIPIENT_DID_REASON_CODE: &str =
    "m2_invalid_owner_recipient_did";
/// Invalid escrow-auditor DID reason marker.
pub const DATA_LAYER_M2_INVALID_AUDITOR_DID_REASON_CODE: &str = "m2_invalid_auditor_did";
