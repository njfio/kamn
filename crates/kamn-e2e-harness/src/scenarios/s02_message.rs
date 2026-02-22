use super::ScenarioDefinition;

/// Returns the S-02 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-02",
        name: "Direct Message Round-Trip",
        priority: "P0",
    }
}
