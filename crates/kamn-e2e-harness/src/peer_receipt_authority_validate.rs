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
    let request = stage(attempt.request.as_ref(), "request")?;
    validate_request(request)?;
    let challenge = stage(attempt.challenge.as_ref(), "challenge")?;
    validate_challenge(request, challenge)?;
    let approval = stage(attempt.approval.as_ref(), "approval")?;
    validate_approval(challenge, approval)?;
    let settlement = stage(attempt.settlement.as_ref(), "settlement")?;
    validate_settlement(approval, settlement)?;
    let result = stage(attempt.service_result.as_ref(), "service_result")?;
    validate_result(request, settlement, result)
}

fn validate_request(value: &PeerRequestAuthority) -> Result<(), PeerReceiptAuthorityError> {
    required("request", "canonical_body", value.canonical_body.as_str())?;
    check_digest(
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
    required_challenge_fields(value)?;
    check_nonzero("challenge", "expires_at_unix", value.expires_at_unix)?;
    check_nonzero("challenge", "amount_minor", value.amount_minor)?;
    binding(
        "challenge",
        "request_digest",
        &request.request_digest,
        &value.request_digest,
    )?;
    check_digest(
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
    required_approval_fields(value)?;
    check_nonzero("approval", "approved_at_unix", value.approved_at_unix)?;
    approval_bindings(challenge, value)?;
    if value.approved_at_unix > challenge.expires_at_unix {
        return Err(time_error("approval", "approved_at_unix"));
    }
    check_digest(
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
    required_settlement_fields(value)?;
    check_nonzero("settlement", "finalized_at_unix", value.finalized_at_unix)?;
    settlement_bindings(approval, value)?;
    if value.finalized_at_unix < approval.approved_at_unix {
        return Err(time_error("settlement", "finalized_at_unix"));
    }
    check_digest(
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
    required_result_fields(value)?;
    check_nonzero("service_result", "produced_at_unix", value.produced_at_unix)?;
    binding(
        "service_result",
        "request_digest",
        &request.request_digest,
        &value.request_digest,
    )?;
    binding(
        "service_result",
        "settlement_digest",
        &settlement.settlement_digest,
        &value.settlement_digest,
    )?;
    if value.produced_at_unix < settlement.finalized_at_unix {
        return Err(time_error("service_result", "produced_at_unix"));
    }
    check_digest(
        "service_result",
        "result_digest",
        value.result_digest.as_str(),
        peer_result_digest(value),
    )
}

fn required_challenge_fields(
    value: &PeerChallengeAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required_fields(
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

fn required_approval_fields(
    value: &PeerApprovalAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required_fields(
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

fn required_settlement_fields(
    value: &PeerSettlementAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    required_fields(
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

fn required_result_fields(value: &PeerResultAuthority) -> Result<(), PeerReceiptAuthorityError> {
    required_fields(
        "service_result",
        &[
            ("request_digest", &value.request_digest),
            ("settlement_digest", &value.settlement_digest),
            ("canonical_result", &value.canonical_result),
            ("result_digest", &value.result_digest),
        ],
    )
}

fn approval_bindings(
    challenge: &PeerChallengeAuthority,
    value: &PeerApprovalAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    compare_fields(
        "approval",
        &[
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
        ],
    )?;
    compare_economics(
        "approval",
        challenge_terms(challenge),
        approval_terms(value),
    )
}

fn settlement_bindings(
    approval: &PeerApprovalAuthority,
    value: &PeerSettlementAuthority,
) -> Result<(), PeerReceiptAuthorityError> {
    compare_fields(
        "settlement",
        &[
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
        ],
    )?;
    compare_economics(
        "settlement",
        approval_terms(approval),
        settlement_terms(value),
    )
}

type Terms<'a> = (&'a str, &'a str, &'a str, &'a str, u64);

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

fn compare_economics(
    stage: &'static str,
    expected: Terms<'_>,
    actual: Terms<'_>,
) -> Result<(), PeerReceiptAuthorityError> {
    for (field, left, right) in [
        ("payer", expected.0, actual.0),
        ("payee", expected.1, actual.1),
        ("asset", expected.2, actual.2),
        ("network", expected.3, actual.3),
    ] {
        binding(stage, field, left, right)?;
    }
    if expected.4 != actual.4 {
        return Err(binding_error(stage, "amount_minor"));
    }
    Ok(())
}

fn compare_fields(
    stage: &'static str,
    fields: &[(&'static str, &String, &String)],
) -> Result<(), PeerReceiptAuthorityError> {
    for (field, expected, actual) in fields {
        binding(stage, field, expected, actual)?;
    }
    Ok(())
}

fn binding(
    stage: &'static str,
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), PeerReceiptAuthorityError> {
    (expected == actual)
        .then_some(())
        .ok_or_else(|| binding_error(stage, field))
}

fn check_digest(
    stage: &'static str,
    field: &'static str,
    claimed: &str,
    computed: String,
) -> Result<(), PeerReceiptAuthorityError> {
    if !valid_digest(claimed) {
        return Err(error("PEER_AUTHORITY_DIGEST_INVALID", stage, field));
    }
    (claimed == computed)
        .then_some(())
        .ok_or_else(|| error("PEER_AUTHORITY_DIGEST_MISMATCH", stage, field))
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn required_fields(
    stage: &'static str,
    fields: &[(&'static str, &String)],
) -> Result<(), PeerReceiptAuthorityError> {
    for (field, value) in fields {
        required(stage, field, value)?;
    }
    Ok(())
}

fn required(
    stage: &'static str,
    field: &'static str,
    value: &str,
) -> Result<(), PeerReceiptAuthorityError> {
    (!value.is_empty())
        .then_some(())
        .ok_or_else(|| error("PEER_AUTHORITY_FIELD_MISSING", stage, field))
}

fn check_nonzero(
    stage: &'static str,
    field: &'static str,
    value: u64,
) -> Result<(), PeerReceiptAuthorityError> {
    (value > 0)
        .then_some(())
        .ok_or_else(|| error("PEER_AUTHORITY_FIELD_MISSING", stage, field))
}

fn stage<'a, T>(
    value: Option<&'a T>,
    stage: &'static str,
) -> Result<&'a T, PeerReceiptAuthorityError> {
    value.ok_or_else(|| error("PEER_AUTHORITY_STAGE_MISSING", stage, "stage"))
}

fn binding_error(stage: &'static str, field: &'static str) -> PeerReceiptAuthorityError {
    error("PEER_AUTHORITY_BINDING_MISMATCH", stage, field)
}

fn time_error(stage: &'static str, field: &'static str) -> PeerReceiptAuthorityError {
    error("PEER_AUTHORITY_TIME_INVALID", stage, field)
}

fn error(
    code: &'static str,
    stage: &'static str,
    field: &'static str,
) -> PeerReceiptAuthorityError {
    PeerReceiptAuthorityError {
        code,
        message: format!("{stage} authority failed at {field}"),
        stage,
        field,
        context: format!("stage={stage},field={field}"),
        cause: None,
    }
}
