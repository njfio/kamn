use super::ScenarioDefinition;

/// Returns the S-11 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-11",
        name: "Signer Key Rotation",
        priority: "P2",
        steps: &[
            "Rotate signer key material",
            "Issue signed request with new key",
            "Verify old key requests are rejected",
        ],
        verifiable_outputs: &[
            "evidence/s11/signer_rotation_event.json",
            "evidence/s11/signer_rotation_validation.json",
        ],
        pass_criteria: &["New signer key is accepted and old key is rejected"],
    }
}
