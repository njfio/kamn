use super::ScenarioDefinition;

/// Returns the S-15 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-15",
        name: "Performance Smoke",
        priority: "P2",
        steps: &[
            "Execute bounded scenario burst",
            "Capture duration and throughput summary",
            "Validate smoke thresholds",
        ],
        verifiable_outputs: &[
            "evidence/s15/performance_smoke_summary.json",
            "evidence/s15/performance_threshold_check.json",
        ],
        pass_criteria: &["Smoke run completes within configured threshold budget"],
    }
}
