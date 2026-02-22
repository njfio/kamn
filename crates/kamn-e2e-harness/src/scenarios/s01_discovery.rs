use super::ScenarioDefinition;

/// Returns the S-01 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-01",
        name: "Agent Discovery & Identity",
        priority: "P0",
    }
}
