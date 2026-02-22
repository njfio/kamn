use super::ScenarioDefinition;

/// Returns the S-09 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-09",
        name: "Transport Failover",
        priority: "P1",
        steps: &[
            "Disconnect primary transport peer route",
            "Trigger fallback route selection",
            "Confirm message delivery continuity",
        ],
        verifiable_outputs: &[
            "evidence/s09/failover_event.json",
            "evidence/s09/failover_delivery_trace.json",
        ],
        pass_criteria: &["Delivery succeeds through fallback transport route"],
    }
}
