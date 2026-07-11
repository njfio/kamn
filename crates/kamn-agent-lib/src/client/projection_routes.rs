use super::*;

impl ServiceApiHttpClient {
    /// Queries the participant-private task projection without rewriting it.
    pub fn get_task_participant_projection(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<String, AgentLibError> {
        Ok(self.inner.get_task_participant_projection(task_id, auth)?)
    }

    /// Queries the restricted-public task projection without rewriting it.
    pub fn get_task_verifier_projection(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<String, AgentLibError> {
        Ok(self.inner.get_task_verifier_projection(task_id, auth)?)
    }
}
