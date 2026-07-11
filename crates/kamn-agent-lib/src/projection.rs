use super::*;

impl KamnAgentHandle {
    /// Queries this agent's participant-private task projection.
    pub fn query_participant_task_projection(
        &self,
        task_id: &str,
    ) -> Result<String, AgentLibError> {
        let auth = self.task_read_auth()?;
        self.service_client
            .get_task_participant_projection(task_id, &auth)
    }

    /// Queries the restricted-public task projection available to this agent.
    pub fn query_verifier_task_projection(&self, task_id: &str) -> Result<String, AgentLibError> {
        let auth = self.task_read_auth()?;
        self.service_client
            .get_task_verifier_projection(task_id, &auth)
    }

    fn task_read_auth(&self) -> Result<kamn_sdk::ServiceRequestAuth, AgentLibError> {
        let nonce = self.next_nonce()?;
        self.service_client.build_auth(
            self.identity.did(),
            self.identity.signing_key(),
            nonce,
            "",
            Some("tasks:read"),
        )
    }
}
