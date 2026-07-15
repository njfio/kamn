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
    if !requires_funding_request(report, &surface)? {
        return Ok(());
    }
    let request = read_funding_request(report)?;
    validate_request(request.as_str(), task_id, digest, escrow_id)
}

fn requires_funding_request(report: &str, surface: &str) -> Result<bool, String> {
    match surface {
        "command-override" if report.contains("\"runtime_agent_a_evidence\":\"") => {
            funding_error("command override cannot satisfy canonical execution".to_owned())
        }
        "command-override" | "live-service-persisted-receipt" => Ok(false),
        "live-service-api" => Ok(true),
        _ => funding_error(format!("unsupported execution surface: {surface}")),
    }
}

fn validate_request(raw: &str, task_id: &str, digest: &str, escrow_id: &str) -> Result<(), String> {
    require_request_string(raw, "schema_version", "kamn.mvp.devnet-settlement.v2")?;
    require_request_string(raw, "task_id", task_id)?;
    require_request_string(raw, "task_binding_digest", digest)?;
    require_request_escrow(raw, escrow_id)
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
