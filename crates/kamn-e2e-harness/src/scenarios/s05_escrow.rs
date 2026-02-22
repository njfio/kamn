use super::ScenarioDefinition;

/// Returns the S-05 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-05",
        name: "Escrow Settlement (Dispute)",
        priority: "P0",
    }
}
