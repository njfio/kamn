use super::ScenarioDefinition;

/// Returns the S-15 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-15",
        name: "Performance Smoke",
        priority: "P2",
    }
}
