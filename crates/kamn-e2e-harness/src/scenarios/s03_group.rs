use super::ScenarioDefinition;

/// Returns the S-03 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-03",
        name: "Group Channel Messaging",
        priority: "P0",
    }
}
