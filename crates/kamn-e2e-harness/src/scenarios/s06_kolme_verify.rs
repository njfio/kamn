use super::ScenarioDefinition;

/// Returns the S-06 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-06",
        name: "Kolme Proof Verification",
        priority: "P0",
        steps: &[
            "Collect message and operation anchors from prior scenarios",
            "Verify each proof via kamn_verify_proof",
            "Query Kolme block data for each anchor",
            "Validate block-hash chain continuity from genesis",
            "Validate processor signatures",
        ],
        verifiable_outputs: &[
            "evidence/s06/kolme_chain_audit.json",
            "evidence/s06/proof_inclusion_map.json",
            "evidence/s06/chain_integrity_report.json",
            "evidence/s06/processor_signatures.json",
        ],
        pass_criteria: &[
            "Every proof maps to a Kolme block anchor",
            "Hash chain continuity is unbroken from genesis",
            "Processor signatures verify successfully",
        ],
    }
}
