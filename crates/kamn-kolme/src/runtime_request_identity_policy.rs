//! Runtime-commit deterministic request identity policy contracts.

/// Renders deterministic idempotency key for a runtime commit request.
pub fn deterministic_runtime_commit_idempotency_key(
    operation_id: &str,
    state_root: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> String {
    format!(
        "kolme-runtime-commit:{}:{}:{}:{}:{}",
        operation_id.trim(),
        state_root.trim(),
        actor_did.trim(),
        nonce,
        payload_hash.trim().len()
    )
}

/// Renders deterministic runtime commit identifier from request fields.
pub fn deterministic_runtime_commit_id(
    operation_id: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
) -> String {
    format!(
        "kolme-commit:{}:{}:{}:{}",
        operation_id,
        actor_did,
        nonce,
        payload_hash.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{deterministic_runtime_commit_id, deterministic_runtime_commit_idempotency_key};

    #[test]
    fn unit_renders_idempotency_key_contract() {
        assert_eq!(
            deterministic_runtime_commit_idempotency_key(
                " operation-123 ",
                " state:abc ",
                " did:kamn:agent:alpha ",
                7,
                " payload-hash "
            ),
            "kolme-runtime-commit:operation-123:state:abc:did:kamn:agent:alpha:7:12"
        );
    }

    #[test]
    fn regression_commit_id_is_payload_length_based() {
        // Regression: #1777
        assert_eq!(
            deterministic_runtime_commit_id("op-x", "did:agent", 3, "abc"),
            deterministic_runtime_commit_id("op-x", "did:agent", 3, "xyz")
        );
    }
}
