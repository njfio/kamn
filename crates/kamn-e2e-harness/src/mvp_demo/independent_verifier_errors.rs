pub(super) fn map_actor_verification_error(error: String) -> String {
    classify_actor_error(error.as_str())
        .map(str::to_owned)
        .unwrap_or(error)
}

pub(super) fn map_three_agent_verification_error(error: String) -> String {
    if is_public_code(error.as_str()) {
        return error;
    }
    let normalized = error.to_ascii_uppercase();
    classify_three_agent_error(normalized.as_str())
        .unwrap_or("TRANSACTION_AGREEMENT_INVALID")
        .to_owned()
}

fn classify_three_agent_error(error: &str) -> Option<&'static str> {
    if contains_any(error, &["VIEW", "PRIVATE", "PROJECTION", "PARTICIPANT"]) {
        return Some("PROJECTION_SCOPE_INVALID");
    }
    if contains_any(error, &["RECEIPT", "RUNTIME CHAIN", "RUNTIME_CHAIN"]) {
        return Some("RECEIPT_CHAIN_INVALID");
    }
    None
}

fn is_public_code(error: &str) -> bool {
    error
        .chars()
        .all(|value| value == '_' || value.is_ascii_uppercase())
}

fn classify_actor_error(error: &str) -> Option<&'static str> {
    if is_identity_error(error) {
        return Some("AGENT_IDENTITY_INVALID");
    }
    if contains_any(error, &["PRIVATE_LEAK", "PROJECTION", "PARTICIPANT_SCOPE"]) {
        return Some("PROJECTION_SCOPE_INVALID");
    }
    if contains_any(error, &["HANDOFF_AUTHORIZATION", "ACTION_NOT_GRANTED"]) {
        return Some("AUTHORIZATION_EVIDENCE_INVALID");
    }
    if contains_any(error, &["CHAIN_FACT_MISMATCH", "TRANSACTION_FACT_MISMATCH"]) {
        return Some("TRANSACTION_AGREEMENT_INVALID");
    }
    (error.contains("RECEIPT_CHAIN") || error.contains("RUNTIME_RECEIPT"))
        .then_some("RECEIPT_CHAIN_INVALID")
}

fn is_identity_error(error: &str) -> bool {
    contains_any(
        error,
        &[
            "IDENTITY",
            "NONCE_STREAM",
            "PROCESS_NOT_DISTINCT",
            "PROCESS_REUSED",
            "DID_NOT_DISTINCT",
        ],
    )
}

fn contains_any(value: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| value.contains(marker))
}
