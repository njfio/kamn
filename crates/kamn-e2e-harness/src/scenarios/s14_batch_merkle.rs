use super::ScenarioDefinition;

/// Returns the S-14 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-14",
        name: "Batch Merkle Anchoring",
        priority: "P2",
        steps: &[
            "Submit batched operations",
            "Build Merkle root for batch",
            "Anchor root and verify inclusion proofs",
        ],
        verifiable_outputs: &[
            "evidence/s14/batch_root.json",
            "evidence/s14/inclusion_proofs.json",
        ],
        pass_criteria: &["All operations verify against anchored Merkle root"],
    }
}
