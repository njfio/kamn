use crate::peer_receipt_authority_validate_bindings as bindings;
use crate::peer_receipt_authority_validate_required as required;
use crate::peer_receipt_authority_validate_support as support;
use crate::{
    peer_approval_digest, peer_challenge_digest, peer_request_digest, peer_result_digest,
    peer_settlement_digest, PeerApprovalAuthority, PeerChallengeAuthority,
    PeerReceiptAuthorityAttempt, PeerReceiptAuthorityError, PeerReceiptAuthorityVerdict,
    PeerRequestAuthority, PeerResultAuthority, PeerSettlementAuthority, PeerSettlementVisibility,
};

/// Validates a complete peer receipt-authority chain.
pub fn verify_peer_receipt_authority(
    attempt: &PeerReceiptAuthorityAttempt,
) -> PeerReceiptAuthorityVerdict {
    match validate(attempt) {
        Ok(()) => PeerReceiptAuthorityVerdict::Pass,
        Err(error) if attempt.settlement_visibility == PeerSettlementVisibility::Blocked => {
            PeerReceiptAuthorityVerdict::Blocked(error)
        }
        Err(error) => PeerReceiptAuthorityVerdict::Fail(error),
    }
}

fn validate(attempt: &PeerReceiptAuthorityAttempt) -> Result<(), PeerReceiptAuthorityError> {
    let request = required::stage(attempt.request.as_ref(), "request")?;
    validate_request(request)?;
    let challenge = required::stage(attempt.challenge.as_ref(), "challenge")?;
    validate_challenge(request, challenge)?;
    let approval = required::stage(attempt.approval.as_ref(), "approval")?;
    validate_approval(challenge, approval)?;
    let settlement = required::stage(attempt.settlement.as_ref(), "settlement")?;
    validate_settlement(approval, settlement)?;
    let result = required::stage(attempt.service_result.as_ref(), "service_result")?;
    validate_result(request, settlement, result)
}

fn validate_request(value: &PeerRequestAuthority) -> Result<(), PeerReceiptAuthorityError> {
    required::text("request", "canonical_body", value.canonical_body.as_str())?;
    support::digest(
        "request",
        "request_digest",
        value.request_digest.as_str(),
        peer_request_digest(value.canonical_body.as_str()),
    )
}

fn validate_challenge(
    request: &PeerRequestAuthority,
    value: &PeerChallengeAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required::challenge(value)?;
    required::nonzero("challenge", "expires_at_unix", value.expires_at_unix)?;
    required::nonzero("challenge", "amount_minor", value.amount_minor)?;
    bindings::field(
        "challenge",
        "request_digest",
        &request.request_digest,
        &value.request_digest,
    )?;
    support::digest(
        "challenge",
        "challenge_digest",
        value.challenge_digest.as_str(),
        peer_challenge_digest(value),
    )
}

fn validate_approval(
    challenge: &PeerChallengeAuthority,
    value: &PeerApprovalAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required::approval(value)?;
    required::nonzero("approval", "approved_at_unix", value.approved_at_unix)?;
    bindings::approval(challenge, value)?;
    if value.approved_at_unix > challenge.expires_at_unix {
        return Err(support::time("approval", "approved_at_unix"));
    }
    support::digest(
        "approval",
        "approval_digest",
        value.approval_digest.as_str(),
        peer_approval_digest(value),
    )
}

fn validate_settlement(
    approval: &PeerApprovalAuthority,
    value: &PeerSettlementAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required::settlement(value)?;
    required::nonzero("settlement", "finalized_at_unix", value.finalized_at_unix)?;
    bindings::settlement(approval, value)?;
    if value.finalized_at_unix < approval.approved_at_unix {
        return Err(support::time("settlement", "finalized_at_unix"));
    }
    support::digest(
        "settlement",
        "settlement_digest",
        value.settlement_digest.as_str(),
        peer_settlement_digest(value),
    )
}

fn validate_result(
    request: &PeerRequestAuthority,
    settlement: &PeerSettlementAuthority,
    value: &PeerResultAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required::result(value)?;
    required::nonzero("service_result", "produced_at_unix", value.produced_at_unix)?;
    validate_result_bindings(request, settlement, value)?;
    if value.produced_at_unix < settlement.finalized_at_unix {
        return Err(support::time("service_result", "produced_at_unix"));
    }
    support::digest(
        "service_result",
        "result_digest",
        value.result_digest.as_str(),
        peer_result_digest(value),
    )
}

fn validate_result_bindings(
    request: &PeerRequestAuthority,
    settlement: &PeerSettlementAuthority,
    value: &PeerResultAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    bindings::field(
        "service_result",
        "request_digest",
        &request.request_digest,
        &value.request_digest,
    )?;
    bindings::field(
        "service_result",
        "settlement_digest",
        &settlement.settlement_digest,
        &value.settlement_digest,
    )
}
