use super::ScenarioDefinition;

/// Returns the S-08 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-08",
        name: "Node Crash Recovery",
        priority: "P1",
        steps: &[
            "Send pre-crash message batch",
            "Crash processor node",
            "Restart processor node",
            "Send post-crash message batch",
            "Validate nonce continuity and no message loss",
        ],
        verifiable_outputs: &[
            "evidence/s08/pre_crash_messages.json",
            "evidence/s08/crash_event.json",
            "evidence/s08/recovery_log.json",
            "evidence/s08/post_crash_messages.json",
            "evidence/s08/state_consistency.json",
        ],
        pass_criteria: &[
            "Zero message loss across crash boundary",
            "Nonce sequence has no gaps or duplicates",
        ],
    }
}
