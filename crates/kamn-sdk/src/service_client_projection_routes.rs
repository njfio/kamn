use super::super::{expect_status, normalize_route_segment, ServiceRequestAuth};
use super::ServiceApiClient;
use crate::SdkError;

impl ServiceApiClient {
    /// Returns the server-generated participant-private task projection JSON.
    pub fn get_task_participant_projection(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<String, SdkError> {
        self.get_task_projection(task_id, "participant-view", auth)
    }

    /// Returns the server-generated restricted-public task projection JSON.
    pub fn get_task_verifier_projection(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<String, SdkError> {
        self.get_task_projection(task_id, "verifier-view", auth)
    }

    fn get_task_projection(
        &self,
        task_id: &str,
        view: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<String, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}/{view}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(response.body)
    }
}
