use super::ScenarioDefinition;

/// Returns the S-09 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-09",
        name: "Transport Failover",
        priority: "P1",
    }
}
