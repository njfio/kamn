use std::collections::HashSet;

use crate::drivers::{
    normalize_authoritative_settlement, AuthoritativeSettlementObservation,
    AuthoritativeSettlementReplayGuard,
};
use crate::settlement_authority_parity_support::{
    build_report, error, identity_error, identity_error_with_code, normalize_error,
    validate_submission_count, AUTHORITY_ERROR, REPLAY_ERROR,
};
use crate::{
    SettlementAuthorityAttempt, SettlementAuthorityDriver, SettlementAuthorityParityError,
    SettlementAuthorityParityReport,
};

/// Verifies complete, identical authority across SDK, CLI, and MCP attempts.
pub fn verify_settlement_authority_parity(
    expected_escrow: &str,
    expected_actor: &str,
    expected_idempotency: &str,
    attempts: Vec<SettlementAuthorityAttempt>,
    settlement_submissions: u64,
) -> Result<SettlementAuthorityParityReport, SettlementAuthorityParityError> {
    validate_driver_set(attempts.as_slice())?;
    let observations = normalize_attempts(
        attempts.as_slice(),
        expected_escrow,
        expected_actor,
        expected_idempotency,
    )?;
    validate_retry_parity(observations.as_slice())?;
    validate_submission_count(settlement_submissions)?;
    build_report(
        expected_escrow,
        expected_idempotency,
        &observations[0].1,
        settlement_submissions,
    )
}

fn validate_driver_set(
    attempts: &[SettlementAuthorityAttempt],
) -> Result<(), SettlementAuthorityParityError> {
    let drivers = attempts
        .iter()
        .map(|attempt| attempt.driver)
        .collect::<HashSet<_>>();
    if attempts.len() == 3 && drivers.len() == 3 {
        return Ok(());
    }
    Err(error(
        AUTHORITY_ERROR,
        None,
        "driver_set",
        "SDK, CLI, and MCP attempts are each required exactly once",
    ))
}

fn normalize_attempts(
    attempts: &[SettlementAuthorityAttempt],
    escrow: &str,
    actor: &str,
    idempotency: &str,
) -> Result<
    Vec<(
        SettlementAuthorityDriver,
        AuthoritativeSettlementObservation,
    )>,
    SettlementAuthorityParityError,
> {
    attempts
        .iter()
        .map(|attempt| normalize_attempt(attempt, escrow, actor, idempotency))
        .collect()
}

fn normalize_attempt(
    attempt: &SettlementAuthorityAttempt,
    escrow: &str,
    actor: &str,
    idempotency: &str,
) -> Result<
    (
        SettlementAuthorityDriver,
        AuthoritativeSettlementObservation,
    ),
    SettlementAuthorityParityError,
> {
    validate_attempt_identity(attempt, escrow, idempotency)?;
    let authority = normalize_authoritative_settlement(&attempt.response, escrow, actor)
        .map_err(|source| normalize_error(attempt.driver, source.as_str()))?;
    if authority.idempotency_key != idempotency {
        return Err(error(
            AUTHORITY_ERROR,
            Some(attempt.driver),
            "idempotency_key",
            "authoritative settlement changed the shared idempotency identity",
        ));
    }
    Ok((attempt.driver, authority))
}

fn validate_attempt_identity(
    attempt: &SettlementAuthorityAttempt,
    escrow: &str,
    idempotency: &str,
) -> Result<(), SettlementAuthorityParityError> {
    if attempt.escrow_id != escrow {
        return Err(identity_error(attempt.driver, "escrow_id"));
    }
    if attempt.idempotency_key != idempotency {
        return Err(identity_error(attempt.driver, "idempotency_key"));
    }
    Ok(())
}

fn validate_retry_parity(
    observations: &[(
        SettlementAuthorityDriver,
        AuthoritativeSettlementObservation,
    )],
) -> Result<(), SettlementAuthorityParityError> {
    let mut guard = AuthoritativeSettlementReplayGuard::default();
    for (driver, authority) in observations {
        guard.observe(authority).map_err(|_| {
            identity_error_with_code(*driver, REPLAY_ERROR, "authoritative_settlement")
        })?;
    }
    Ok(())
}
