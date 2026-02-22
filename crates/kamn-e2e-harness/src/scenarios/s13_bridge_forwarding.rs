use super::ScenarioDefinition;

/// Returns the S-13 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-13",
        name: "Bridge Message Forwarding",
        priority: "P2",
        steps: &[
            "Submit source-network bridge message",
            "Forward message through bridge adapter",
            "Confirm target-network receipt",
        ],
        verifiable_outputs: &[
            "evidence/s13/bridge_forward_request.json",
            "evidence/s13/bridge_forward_receipt.json",
        ],
        pass_criteria: &["Bridge forwarded payload appears on target network once"],
    }
}
