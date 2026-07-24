use crate::peer_receipt_authority_validate_support::error;
use crate::{
    PeerApprovalAuthority, PeerChallengeAuthority, PeerReceiptAuthorityError,
    PeerSettlementAuthority,
};

type Terms<'a> = (&'a str, &'a str, &'a str, &'a str, u64);

pub(crate) fn approval(
    challenge: &PeerChallengeAuthority,
    value: &PeerApprovalAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    fields("approval", &approval_fields(challenge, value))?;
    economics(
        "approval",
        challenge_terms(challenge),
        approval_terms(value),
    )
}

pub(crate) fn settlement(
    approval: &PeerApprovalAuthority,
    value: &PeerSettlementAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    fields("settlement", &settlement_fields(approval, value))?;
    economics(
        "settlement",
        approval_terms(approval),
        settlement_terms(value),
    )
}

fn approval_fields<'a>(
    challenge: &'a PeerChallengeAuthority,
    value: &'a PeerApprovalAuthority,
) -> [(&'static str, &'a String, &'a String); 4] {
    [
        (
            "request_digest",
            &challenge.request_digest,
            &value.request_digest,
        ),
        (
            "challenge_digest",
            &challenge.challenge_digest,
            &value.challenge_digest,
        ),
        ("challenge_id", &challenge.challenge_id, &value.challenge_id),
        ("nonce", &challenge.nonce, &value.nonce),
    ]
}

fn settlement_fields<'a>(
    approval: &'a PeerApprovalAuthority,
    value: &'a PeerSettlementAuthority,
) -> [(&'static str, &'a String, &'a String); 3] {
    [
        (
            "request_digest",
            &approval.request_digest,
            &value.request_digest,
        ),
        (
            "challenge_digest",
            &approval.challenge_digest,
            &value.challenge_digest,
        ),
        (
            "approval_digest",
            &approval.approval_digest,
            &value.approval_digest,
        ),
    ]
}

pub(crate) fn field(
    stage: &'static str,
    name: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), PeerReceiptAuthorityError> {
    (expected == actual)
        .then_some(())
        .ok_or_else(|| mismatch(stage, name))
}

fn fields(
    stage: &'static str,
    values: &[(&'static str, &String, &String)],
) -> Result<(), PeerReceiptAuthorityError> {
    for (name, expected, actual) in values {
        field(stage, name, expected, actual)?;
    }
    Ok(())
}

fn economics(
    stage: &'static str,
    expected: Terms<'_>,
    actual: Terms<'_>,
) -> Result<(), PeerReceiptAuthorityError> {
    for (name, left, right) in [
        ("payer", expected.0, actual.0),
        ("payee", expected.1, actual.1),
        ("asset", expected.2, actual.2),
        ("network", expected.3, actual.3),
    ] {
        field(stage, name, left, right)?;
    }
    (expected.4 == actual.4)
        .then_some(())
        .ok_or_else(|| mismatch(stage, "amount_minor"))
}

fn challenge_terms(value: &PeerChallengeAuthority) -> Terms<'_> {
    (
        &value.payer,
        &value.payee,
        &value.asset,
        &value.network,
        value.amount_minor,
    )
}

fn approval_terms(value: &PeerApprovalAuthority) -> Terms<'_> {
    (
        &value.payer,
        &value.payee,
        &value.asset,
        &value.network,
        value.amount_minor,
    )
}

fn settlement_terms(value: &PeerSettlementAuthority) -> Terms<'_> {
    (
        &value.payer,
        &value.payee,
        &value.asset,
        &value.network,
        value.amount_minor,
    )
}

fn mismatch(stage: &'static str, field: &'static str) -> PeerReceiptAuthorityError {
    error("PEER_AUTHORITY_BINDING_MISMATCH", stage, field)
}
