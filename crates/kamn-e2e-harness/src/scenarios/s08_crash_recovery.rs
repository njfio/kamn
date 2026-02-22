use super::ScenarioDefinition;

/// Returns the S-08 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-08",
        name: "Node Crash Recovery",
        priority: "P1",
    }
}
