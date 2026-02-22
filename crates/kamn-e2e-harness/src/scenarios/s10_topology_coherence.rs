use super::ScenarioDefinition;

/// Returns the S-10 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-10",
        name: "Multi-Node Topology Coherence",
        priority: "P1",
    }
}
