use super::ScenarioDefinition;

/// Returns the S-04 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-04",
        name: "Task Lifecycle (Full)",
        priority: "P0",
    }
}
