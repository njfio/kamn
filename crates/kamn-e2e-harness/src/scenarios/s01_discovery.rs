use super::ScenarioDefinition;

/// Returns the S-01 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-01",
        name: "Agent Discovery & Identity",
        priority: "P0",
        steps: &[
            "Alice registers DID via kamn_register",
            "Bob registers DID via kamn_register",
            "Alice creates direct channel with Bob via kamn_create_channel",
            "Orchestrator verifies peer/service health before scenario close",
        ],
        verifiable_outputs: &[
            "evidence/s01/alice_registration.json",
            "evidence/s01/bob_registration.json",
            "evidence/s01/channel_create_receipt.json",
            "evidence/s01/kolme_anchor_block.json",
        ],
        pass_criteria: &[
            "Alice and Bob DID registrations are present",
            "Direct channel creation returns member pair Alice/Bob",
            "Registration/channel anchors report FINAL finality",
        ],
    }
}
