use super::ScenarioDefinition;

/// Returns the S-13 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-13",
        name: "Bridge Message Forwarding",
        priority: "P2",
    }
}
