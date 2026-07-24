use crate::peer_receipt_authority_validate_support::error;
use crate::{
    PeerApprovalAuthority, PeerChallengeAuthority, PeerReceiptAuthorityError, PeerResultAuthority,
    PeerSettlementAuthority,
};

pub(crate) fn challenge(value: &PeerChallengeAuthority) -> Result<(), PeerReceiptAuthorityError> {
    fields(
        "challenge",
        &[
            ("request_digest", &value.request_digest),
            ("challenge_id", &value.challenge_id),
            ("nonce", &value.nonce),
            ("payer", &value.payer),
            ("payee", &value.payee),
            ("asset", &value.asset),
            ("network", &value.network),
            ("challenge_digest", &value.challenge_digest),
        ],
    )
}

pub(crate) fn approval(value: &PeerApprovalAuthority) -> Result<(), PeerReceiptAuthorityError> {
    fields(
        "approval",
        &[
            ("request_digest", &value.request_digest),
            ("challenge_digest", &value.challenge_digest),
            ("challenge_id", &value.challenge_id),
            ("nonce", &value.nonce),
            ("payer", &value.payer),
            ("payee", &value.payee),
            ("asset", &value.asset),
            ("network", &value.network),
            ("approval_digest", &value.approval_digest),
        ],
    )
}

pub(crate) fn settlement(value: &PeerSettlementAuthority) -> Result<(), PeerReceiptAuthorityError> {
    fields(
        "settlement",
        &[
            ("request_digest", &value.request_digest),
            ("challenge_digest", &value.challenge_digest),
            ("approval_digest", &value.approval_digest),
            ("receipt_id", &value.receipt_id),
            ("transaction_id", &value.transaction_id),
            ("payer", &value.payer),
            ("payee", &value.payee),
            ("asset", &value.asset),
            ("network", &value.network),
            ("settlement_digest", &value.settlement_digest),
        ],
    )
}

pub(crate) fn result(value: &PeerResultAuthority) -> Result<(), PeerReceiptAuthorityError> {
    fields(
        "service_result",
        &[
            ("request_digest", &value.request_digest),
            ("settlement_digest", &value.settlement_digest),
            ("canonical_result", &value.canonical_result),
            ("result_digest", &value.result_digest),
        ],
    )
}

pub(crate) fn text(
    stage: &'static str,
    field: &'static str,
    value: &str,
) -> Result<(), PeerReceiptAuthorityError> {
    (!value.is_empty())
        .then_some(())
        .ok_or_else(|| error("PEER_AUTHORITY_FIELD_MISSING", stage, field))
}

pub(crate) fn nonzero(
    stage: &'static str,
    field: &'static str,
    value: u64,
) -> Result<(), PeerReceiptAuthorityError> {
    (value > 0)
        .then_some(())
        .ok_or_else(|| error("PEER_AUTHORITY_FIELD_MISSING", stage, field))
}

pub(crate) fn stage<'a, T>(
    value: Option<&'a T>,
    stage: &'static str,
) -> Result<&'a T, PeerReceiptAuthorityError> {
    value.ok_or_else(|| error("PEER_AUTHORITY_STAGE_MISSING", stage, "stage"))
}

fn fields(
    stage: &'static str,
    values: &[(&'static str, &String)],
) -> Result<(), PeerReceiptAuthorityError> {
    for (field, value) in values {
        text(stage, field, value)?;
    }
    Ok(())
}
