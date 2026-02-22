use super::ScenarioDefinition;

/// Returns the S-11 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-11",
        name: "Signer Key Rotation",
        priority: "P2",
    }
}
