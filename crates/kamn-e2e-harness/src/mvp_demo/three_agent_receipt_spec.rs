use super::artifact_digest::ThreeAgentViewDigests;
use super::three_agent_views::{agent_a_private_view_digest, agent_b_private_view_digest};

#[derive(Clone, Copy)]
pub(super) struct ReceiptSpec {
    pub(super) agent: &'static str,
    pub(super) action: &'static str,
    pub(super) context: &'static str,
    pub(super) artifact_entry: &'static str,
    pub(super) artifact_field: &'static str,
    pub(super) view_file: &'static str,
    pub(super) view_field: &'static str,
    pub(super) view_digest_field: &'static str,
    pub(super) private_digest_field: &'static str,
    pub(super) public_digest_field: &'static str,
    pub(super) digest_field: &'static str,
}

impl ReceiptSpec {
    pub(super) fn view_digest(self, views: &ThreeAgentViewDigests) -> &str {
        match self.agent {
            "agent_a" => views.agent_a.as_str(),
            _ => views.agent_b.as_str(),
        }
    }

    pub(super) fn private_digest(self, run_id: &str) -> String {
        match self.agent {
            "agent_a" => agent_a_private_view_digest(run_id),
            _ => agent_b_private_view_digest(run_id),
        }
    }
}

pub(super) fn agent_a_spec() -> ReceiptSpec {
    ReceiptSpec {
        agent: "agent_a",
        action: "register_and_invoke_transaction",
        context: "agent_a_observation_receipt",
        artifact_entry: "agent_a_observation_receipt",
        artifact_field: "agent_a_observation_receipt_artifact",
        view_file: "agent-a-view.json",
        view_field: "agent_a_view_artifact",
        view_digest_field: "agent_a_view_digest",
        private_digest_field: "agent_a_private_view_digest",
        public_digest_field: "agent_a_public_view_digest",
        digest_field: "agent_a_observation_receipt_digest",
    }
}

pub(super) fn agent_b_spec() -> ReceiptSpec {
    ReceiptSpec {
        agent: "agent_b",
        action: "register_and_accept_task",
        context: "agent_b_observation_receipt",
        artifact_entry: "agent_b_observation_receipt",
        artifact_field: "agent_b_observation_receipt_artifact",
        view_file: "agent-b-view.json",
        view_field: "agent_b_view_artifact",
        view_digest_field: "agent_b_view_digest",
        private_digest_field: "agent_b_private_view_digest",
        public_digest_field: "agent_b_public_view_digest",
        digest_field: "agent_b_observation_receipt_digest",
    }
}
