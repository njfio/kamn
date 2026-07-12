pub(super) fn map_actor_verification_error(error: String) -> String {
    if contains_any(
        error.as_str(),
        &[
            "IDENTITY",
            "NONCE_STREAM",
            "PROCESS_NOT_DISTINCT",
            "DID_NOT_DISTINCT",
        ],
    ) {
        return "AGENT_IDENTITY_INVALID".to_owned();
    }
    if contains_any(
        error.as_str(),
        &["PRIVATE_LEAK", "PROJECTION", "PARTICIPANT_SCOPE"],
    ) {
        return "PROJECTION_SCOPE_INVALID".to_owned();
    }
    if contains_any(
        error.as_str(),
        &["HANDOFF_AUTHORIZATION", "ACTION_NOT_GRANTED"],
    ) {
        return "AUTHORIZATION_EVIDENCE_INVALID".to_owned();
    }
    if error.contains("FACT_MISMATCH") {
        return "TRANSACTION_AGREEMENT_INVALID".to_owned();
    }
    if error.contains("RECEIPT_CHAIN") || error.contains("RUNTIME_RECEIPT") {
        return "RECEIPT_CHAIN_INVALID".to_owned();
    }
    error
}

fn contains_any(value: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| value.contains(marker))
}
