pub const BASELINE_SIGNATURE_PROFILE_ID: &str = "baseline-v1";

pub fn baseline_signature_profile_id() -> &'static str {
    BASELINE_SIGNATURE_PROFILE_ID
}

pub fn baseline_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{}:{}:{}:{}:{}",
        baseline_signature_profile_id(),
        sender,
        nonce,
        state_hash,
        payload.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        baseline_signature_for_fields, baseline_signature_profile_id, BASELINE_SIGNATURE_PROFILE_ID,
    };

    #[test]
    fn baseline_signature_profile_is_deterministic() {
        let signature_a = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
        let signature_b = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
        assert_eq!(signature_a, signature_b);
    }

    #[test]
    fn baseline_signature_profile_includes_nonce_and_payload_length() {
        let signature = baseline_signature_for_fields("agent-a", 9, "state:x", "abcdef");
        assert_eq!(signature, "sig:baseline-v1:agent-a:9:state:x:6");
    }

    #[test]
    fn baseline_signature_profile_id_helper_matches_constant() {
        assert_eq!(
            baseline_signature_profile_id(),
            BASELINE_SIGNATURE_PROFILE_ID
        );
    }
}
