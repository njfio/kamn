use super::ScenarioDefinition;

/// Returns the S-14 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-14",
        name: "Batch Merkle Anchoring",
        priority: "P2",
    }
}
