use super::ScenarioDefinition;

/// Returns the S-06 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-06",
        name: "Kolme Proof Verification",
        priority: "P0",
    }
}
