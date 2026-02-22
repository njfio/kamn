use super::ScenarioDefinition;

/// Returns the S-07 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-07",
        name: "Message Replay Protection",
        priority: "P1",
        steps: &[
            "Submit baseline signed message request",
            "Replay identical nonce/signature request",
            "Confirm replay request is rejected deterministically",
        ],
        verifiable_outputs: &[
            "evidence/s07/replay_attempt.json",
            "evidence/s07/replay_rejection.json",
        ],
        pass_criteria: &["Replay request is rejected with deterministic replay guard error"],
    }
}
