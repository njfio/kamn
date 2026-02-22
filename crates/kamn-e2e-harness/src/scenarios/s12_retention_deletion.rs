use super::ScenarioDefinition;

/// Returns the S-12 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-12",
        name: "Content Retention & Deletion",
        priority: "P2",
    }
}
