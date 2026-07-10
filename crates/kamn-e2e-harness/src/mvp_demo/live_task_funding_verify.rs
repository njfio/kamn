use super::devnet_settlement_service::expected_escrow_id;
use super::verify_support::{extract_string, ClaimView};

pub(super) fn validate_funding_request(
    report: &str,
    devnet: &ClaimView<'_>,
    task_id: &str,
    digest: &str,
    escrow_id: &str,
) -> Result<(), String> {
    let surface = extract_string(devnet.raw, "execution_surface")?;
    if surface == "command-override" {
        return Ok(());
    }
    if surface != "live-service-api" {
        return funding_error(format!("unsupported execution surface: {surface}"));
    }
    let request = read_funding_request(report)?;
    require_request_string(
        request.as_str(),
        "schema_version",
        "kamn.mvp.devnet-settlement.v2",
    )?;
    require_request_string(request.as_str(), "task_id", task_id)?;
    require_request_string(request.as_str(), "task_binding_digest", digest)?;
    require_request_escrow(request.as_str(), escrow_id)
}

fn read_funding_request(report: &str) -> Result<String, String> {
    let path = extract_string(report, "devnet_escrow_funding_request")
        .map_err(|error| format!("devnet escrow funding request artifact missing: {error}"))?;
    std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read devnet escrow funding request: {error}"))
}

fn require_request_string(raw: &str, field: &str, expected: &str) -> Result<(), String> {
    match extract_string(raw, field) {
        Ok(actual) if actual == expected => Ok(()),
        Ok(_) => funding_error(format!("{field} mismatch")),
        Err(error) => funding_error(error),
    }
}

fn require_request_escrow(raw: &str, escrow_id: &str) -> Result<(), String> {
    let expected = expected_escrow_id(raw);
    if escrow_id == expected {
        Ok(())
    } else {
        funding_error(format!(
            "escrow ID mismatch: expected {expected}, found {escrow_id}"
        ))
    }
}

fn funding_error<T>(detail: String) -> Result<T, String> {
    Err(format!("devnet escrow funding request {detail}"))
}
