use super::ScenarioDefinition;

/// Returns the S-03 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-03",
        name: "Group Channel Messaging",
        priority: "P0",
        steps: &[
            "Alice creates group channel with Bob and Carol",
            "Alice sends group message",
            "Bob and Carol list messages and receive delivery",
            "Bob replies and Alice/Carol receive reply",
            "Carol is removed and post-removal message is isolated",
        ],
        verifiable_outputs: &[
            "evidence/s03/channel_create_receipt.json",
            "evidence/s03/group_deliveries.json",
            "evidence/s03/membership_change.json",
            "evidence/s03/post_removal_isolation.json",
        ],
        pass_criteria: &[
            "All members receive group messages before removal",
            "Post-removal message excludes removed member",
            "Membership change is auditable in scenario evidence",
        ],
    }
}
