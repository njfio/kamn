use super::ScenarioDefinition;

/// Returns the S-02 scenario definition.
pub fn definition() -> ScenarioDefinition {
    ScenarioDefinition {
        id: "S-02",
        name: "Direct Message Round-Trip",
        priority: "P0",
        steps: &[
            "Alice sends encrypted direct message to Bob via kamn_send_message",
            "Bob queries message lifecycle via kamn_query_message",
            "Bob sends reply message to Alice",
            "Both participants verify proof anchors via kamn_verify_proof",
        ],
        verifiable_outputs: &[
            "evidence/s02/message_send_receipt.json",
            "evidence/s02/message_delivery_trace.json",
            "evidence/s02/reply_receipt.json",
            "evidence/s02/kolme_message_anchor.json",
        ],
        pass_criteria: &[
            "Both direct messages transition to DELIVERED",
            "Envelope lifecycle includes Signed and Encrypted states",
            "Kolme proof finality is FINAL for message anchors",
        ],
    }
}
