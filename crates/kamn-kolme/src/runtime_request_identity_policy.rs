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

/// Validates runtime finality request commit identifier input.
pub fn is_valid_runtime_commit_id_request(commit_id: &str) -> bool {
    !commit_id.trim().is_empty()
}

/// Validates runtime commit request operation identifier input.
pub fn is_valid_runtime_operation_id_input(operation_id: &str) -> bool {
    !operation_id.trim().is_empty()
}

/// Validates runtime commit request state-root input.
pub fn is_valid_runtime_state_root_input(state_root: &str) -> bool {
    !state_root.trim().is_empty()
}

/// Validates runtime commit request payload-hash input.
pub fn is_valid_runtime_payload_hash_input(payload_hash: &str) -> bool {
    !payload_hash.trim().is_empty()
}

/// Returns whether runtime commit request fields satisfy single-line constraints.
pub fn are_runtime_commit_request_fields_single_line(
    operation_id: &str,
    state_root: &str,
    payload_hash: &str,
) -> bool {
    !operation_id.contains('\n') && !state_root.contains('\n') && !payload_hash.contains('\n')
}

#[cfg(test)]
mod tests {
    use super::{
        are_runtime_commit_request_fields_single_line, deterministic_runtime_commit_id,
        deterministic_runtime_commit_idempotency_key, is_valid_runtime_commit_id_request,
        is_valid_runtime_operation_id_input, is_valid_runtime_payload_hash_input,
        is_valid_runtime_state_root_input,
    };

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

    #[test]
    fn unit_validates_runtime_commit_id_request_input() {
        assert!(is_valid_runtime_commit_id_request(
            "kolme-commit:op:did:1:12"
        ));
        assert!(!is_valid_runtime_commit_id_request("   "));
    }

    #[test]
    fn unit_validates_runtime_commit_request_field_inputs() {
        assert!(is_valid_runtime_operation_id_input("op-123"));
        assert!(is_valid_runtime_state_root_input("state:abc"));
        assert!(is_valid_runtime_payload_hash_input("payload:hash"));
        assert!(!is_valid_runtime_operation_id_input(" "));
        assert!(!is_valid_runtime_state_root_input(" "));
        assert!(!is_valid_runtime_payload_hash_input(" "));
    }

    #[test]
    fn unit_validates_runtime_commit_request_fields_single_line() {
        assert!(are_runtime_commit_request_fields_single_line(
            "op-123",
            "state:abc",
            "payload:hash"
        ));
        assert!(!are_runtime_commit_request_fields_single_line(
            "op-123\nwrapped",
            "state:abc",
            "payload:hash"
        ));
    }
}
