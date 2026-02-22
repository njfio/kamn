use super::ScenarioDefinition;

/// Returns the S-07 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-07",
        name: "Message Replay Protection",
        priority: "P1",
    }
}
