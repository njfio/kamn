use super::ScenarioDefinition;

/// Returns the S-05 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-05",
        name: "Escrow Settlement",
        priority: "P0",
        steps: &[
            "Alice funds escrow for Bob",
            "Alice performs partial release",
            "Alice opens dispute on remaining amount",
            "Carol resolves dispute allocation",
            "All transitions are verified via kamn_verify_proof",
        ],
        verifiable_outputs: &[
            "evidence/s05/escrow_lifecycle_trace.json",
            "evidence/s05/settlement_breakdown.json",
            "evidence/s05/dispute_resolution.json",
            "evidence/s05/kolme_escrow_anchors.json",
        ],
        pass_criteria: &[
            "Final balances match dispute resolution allocation",
            "All escrow transitions have anchored proofs",
            "Mediator authorization is required for dispute resolution",
        ],
    }
}
