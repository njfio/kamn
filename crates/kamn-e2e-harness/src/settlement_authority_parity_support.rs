use crate::drivers::AuthoritativeSettlementObservation;
use crate::{
    SettlementAuthorityDriver, SettlementAuthorityParityError, SettlementAuthorityParityReport,
};

pub(crate) const AUTHORITY_ERROR: &str = "PI_SERVICE_AUTHORITY_MISMATCH";
const CHAIN_ERROR: &str = "RECEIPT_CHAIN_INVALID";
pub(crate) const REPLAY_ERROR: &str = "SERVICE_AUTHORITY_REPLAY";

pub(crate) fn validate_submission_count(count: u64) -> Result<(), SettlementAuthorityParityError> {
    if count == 1 {
        return Ok(());
    }
    Err(error(
        REPLAY_ERROR,
        None,
        "settlement_submissions",
        format!("expected one settlement submission, observed {count}"),
    ))
}

pub(crate) fn build_report(
    escrow: &str,
    idempotency: &str,
    authority: &AuthoritativeSettlementObservation,
    count: u64,
) -> Result<SettlementAuthorityParityReport, SettlementAuthorityParityError> {
    let canonical_authority = canonical_authority(authority)?;
    Ok(SettlementAuthorityParityReport {
        escrow_id: escrow.to_owned(),
        idempotency_key: idempotency.to_owned(),
        canonical_authority,
        settlement_submissions: count,
    })
}

fn canonical_authority(
    authority: &AuthoritativeSettlementObservation,
) -> Result<String, SettlementAuthorityParityError> {
    serde_json::to_value(authority)
        .and_then(|value| serde_json::to_string(&value))
        .map_err(|source| {
            error(
                AUTHORITY_ERROR,
                None,
                "authoritative_settlement",
                format!("failed to serialize normalized authority: {source}"),
            )
        })
}

pub(crate) fn normalize_error(
    driver: SettlementAuthorityDriver,
    source: &str,
) -> SettlementAuthorityParityError {
    let code = if source == "SERVICE_AUTHORITY_DIGEST_INVALID" {
        CHAIN_ERROR
    } else {
        AUTHORITY_ERROR
    };
    error(code, Some(driver), "authoritative_settlement", source)
}

pub(crate) fn identity_error(
    driver: SettlementAuthorityDriver,
    field: &'static str,
) -> SettlementAuthorityParityError {
    identity_error_with_code(driver, AUTHORITY_ERROR, field)
}

pub(crate) fn identity_error_with_code(
    driver: SettlementAuthorityDriver,
    code: &'static str,
    field: &'static str,
) -> SettlementAuthorityParityError {
    error(code, Some(driver), field, format!("driver changed {field}"))
}

pub(crate) fn error(
    code: &'static str,
    driver: Option<SettlementAuthorityDriver>,
    field: &'static str,
    message: impl Into<String>,
) -> SettlementAuthorityParityError {
    SettlementAuthorityParityError::new(code, driver, field, message)
}
