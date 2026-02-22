use super::ScenarioDefinition;

/// Returns the S-04 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-04",
        name: "Task Lifecycle (Full)",
        priority: "P0",
        steps: &[
            "Alice creates task definition via kamn_create_task",
            "Alice funds escrow via kamn_fund_escrow",
            "Bob accepts task via kamn_accept_task",
            "Bob completes task with evidence via kamn_complete_task",
            "Alice releases escrow via kamn_release_escrow",
            "Participants verify settlement anchor via kamn_verify_proof",
        ],
        verifiable_outputs: &[
            "evidence/s04/task_create_receipt.json",
            "evidence/s04/task_lifecycle_trace.json",
            "evidence/s04/escrow_fund_receipt.json",
            "evidence/s04/escrow_release_receipt.json",
            "evidence/s04/kolme_settlement_anchor.json",
        ],
        pass_criteria: &[
            "Lifecycle transitions Pending -> Accepted -> Completed",
            "Escrow transitions Funded -> Released",
            "Kolme finality remains FINAL for settlement",
        ],
    }
}
