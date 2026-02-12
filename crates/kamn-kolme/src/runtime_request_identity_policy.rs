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

/// Renders runtime commit wire payload in canonical field order.
pub fn render_runtime_commit_wire_payload(
    operation_id: &str,
    state_root: &str,
    actor_did: &str,
    nonce: u64,
    payload_hash: &str,
    idempotency_key: &str,
) -> String {
    format!(
        "operation_id={}\nstate_root={}\nactor_did={}\nnonce={}\npayload_hash={}\nidempotency_key={}\n",
        operation_id, state_root, actor_did, nonce, payload_hash, idempotency_key
    )
}

/// Normalizes runtime commit request fields before deterministic identity/rendering.
pub fn normalize_runtime_commit_request_fields(
    operation_id: &str,
    state_root: &str,
    payload_hash: &str,
) -> (String, String, String) {
    (
        operation_id.trim().to_owned(),
        state_root.trim().to_owned(),
        payload_hash.trim().to_owned(),
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

/// Validates runtime commit request nonce input.
pub fn is_valid_runtime_nonce_input(nonce: u64) -> bool {
    nonce > 0
}

/// Returns whether runtime commit request fields satisfy single-line constraints.
pub fn are_runtime_commit_request_fields_single_line(
    operation_id: &str,
    state_root: &str,
    payload_hash: &str,
) -> bool {
    !operation_id.contains('\n') && !state_root.contains('\n') && !payload_hash.contains('\n')
}

/// Returns whether the signed message equals the canonical runtime commit wire payload.
pub fn is_canonical_runtime_commit_signed_message(
    canonical_message: &str,
    signed_message: &str,
) -> bool {
    signed_message == canonical_message
}

/// Normalizes signed-envelope fields for deterministic runtime commit wiring.
pub fn normalize_runtime_commit_signed_envelope_fields(
    signer_key_id: &str,
    signed_message: &str,
    signature: &str,
) -> (String, String, String) {
    (
        signer_key_id.trim().to_owned(),
        signed_message.trim().to_owned(),
        signature.trim().to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        are_runtime_commit_request_fields_single_line, deterministic_runtime_commit_id,
        deterministic_runtime_commit_idempotency_key, is_canonical_runtime_commit_signed_message,
        is_valid_runtime_commit_id_request, is_valid_runtime_nonce_input,
        is_valid_runtime_operation_id_input, is_valid_runtime_payload_hash_input,
        is_valid_runtime_state_root_input, normalize_runtime_commit_request_fields,
        normalize_runtime_commit_signed_envelope_fields, render_runtime_commit_wire_payload,
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
    fn unit_validates_runtime_commit_request_nonce_input() {
        assert!(is_valid_runtime_nonce_input(1));
        assert!(!is_valid_runtime_nonce_input(0));
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

    #[test]
    fn unit_validates_canonical_runtime_commit_signed_message_match() {
        assert!(is_canonical_runtime_commit_signed_message(
            "operation_id=op-1\nstate_root=state-1\n",
            "operation_id=op-1\nstate_root=state-1\n",
        ));
        assert!(!is_canonical_runtime_commit_signed_message(
            "operation_id=op-1\nstate_root=state-1\n",
            "operation_id=op-2\nstate_root=state-1\n",
        ));
    }

    #[test]
    fn unit_normalizes_runtime_commit_signed_envelope_fields() {
        let normalized = normalize_runtime_commit_signed_envelope_fields(
            " signer-key-1 ",
            " canonical-payload ",
            " signature-hex ",
        );
        assert_eq!(
            normalized,
            (
                "signer-key-1".to_owned(),
                "canonical-payload".to_owned(),
                "signature-hex".to_owned(),
            )
        );
    }

    #[test]
    fn unit_renders_runtime_commit_wire_payload_contract() {
        let payload = render_runtime_commit_wire_payload(
            "op-11",
            "state:gamma",
            "did:kamn:agent:gamma",
            4,
            "payload:gamma",
            "idempotency:gamma",
        );
        assert_eq!(
            payload,
            "operation_id=op-11\nstate_root=state:gamma\nactor_did=did:kamn:agent:gamma\nnonce=4\npayload_hash=payload:gamma\nidempotency_key=idempotency:gamma\n"
        );
    }

    #[test]
    fn unit_normalizes_runtime_commit_request_fields() {
        let normalized = normalize_runtime_commit_request_fields(
            " operation-42 ",
            " state:epsilon ",
            " payload:epsilon ",
        );
        assert_eq!(
            normalized,
            (
                "operation-42".to_owned(),
                "state:epsilon".to_owned(),
                "payload:epsilon".to_owned(),
            )
        );
    }
}
