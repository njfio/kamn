use super::ScenarioDefinition;

/// Returns the S-12 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-12",
        name: "Content Retention & Deletion",
        priority: "P2",
        steps: &[
            "Create retained content record",
            "Apply retention policy expiration",
            "Execute deletion workflow and verify redaction markers",
        ],
        verifiable_outputs: &[
            "evidence/s12/retention_policy_trace.json",
            "evidence/s12/deletion_audit.json",
        ],
        pass_criteria: &["Expired content is deleted according to retention policy"],
    }
}
