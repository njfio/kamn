use super::report::{
    CLAIM_LABEL_DEVNET_BACKED, CLAIM_LABEL_DRY_RUN, CLAIM_LABEL_LOCAL_ONLY,
    CLAIM_LABEL_PLACEHOLDER, CLAIM_LABEL_REAL, CLAIM_LABEL_ROADMAP,
};
use super::verify_support::ClaimView;

const REQUIRED_CLAIMS: &[&str] = &[
    "local_runtime_startup",
    "authenticated_agent_identities",
    "signed_message_or_task_flow",
    "durable_state_written",
    "relay_projection_visible",
    "websocket_event_visibility",
    "audit_proof_export",
];

const VALUE_TERMS: &[&str] = &[
    "exchange",
    "escrow",
    "settlement",
    "transfer",
    "lamports",
    "asset",
    "value movement",
];

pub(super) fn validate_required_claims(claims: &[ClaimView<'_>]) -> Result<(), String> {
    for required in REQUIRED_CLAIMS {
        if !claims.iter().any(|claim| claim.id == *required) {
            return Err(format!("missing required MVP claim: {required}"));
        }
    }
    Ok(())
}

pub(super) fn validate_claim_label(label: &str) -> Result<(), String> {
    if allowed_labels().contains(&label) {
        return Ok(());
    }
    Err(format!("unknown MVP claim label: {label}"))
}

pub(super) fn validate_authoritative_label(claim: &ClaimView<'_>) -> Result<(), String> {
    if invalid_three_agent_label(claim) {
        return Err("AGENT_TRANSACTION_CLAIM_INVALID".to_owned());
    }
    match claim.label.as_str() {
        CLAIM_LABEL_DRY_RUN => Err("required MVP claim cannot be dry-run".to_owned()),
        CLAIM_LABEL_PLACEHOLDER => Err("required MVP claim cannot be placeholder".to_owned()),
        _ => Ok(()),
    }
}

pub(super) fn validate_value_movement_label(claim: &ClaimView<'_>) -> Result<(), String> {
    if !mentions_value_movement(claim.raw) || claim.label == CLAIM_LABEL_DEVNET_BACKED {
        return Ok(());
    }
    Err("value movement claim must be devnet-backed".to_owned())
}

fn invalid_three_agent_label(claim: &ClaimView<'_>) -> bool {
    claim.id == "three_agent_escrow_verification"
        && matches!(
            claim.label.as_str(),
            CLAIM_LABEL_DRY_RUN | CLAIM_LABEL_PLACEHOLDER | CLAIM_LABEL_LOCAL_ONLY
        )
}

fn allowed_labels() -> [&'static str; 6] {
    [
        CLAIM_LABEL_REAL,
        CLAIM_LABEL_DEVNET_BACKED,
        CLAIM_LABEL_LOCAL_ONLY,
        CLAIM_LABEL_DRY_RUN,
        CLAIM_LABEL_PLACEHOLDER,
        CLAIM_LABEL_ROADMAP,
    ]
}

fn mentions_value_movement(raw: &str) -> bool {
    let lowercase = raw.to_ascii_lowercase();
    VALUE_TERMS.iter().any(|term| lowercase.contains(term))
}
