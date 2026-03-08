use super::{
    config::{CONTENT_WRITE_SCOPE, ESCROW_WRITE_SCOPE, MESSAGES_WRITE_SCOPE, TASKS_WRITE_SCOPE},
    state::{build_auth, remember_message_id},
    task_escrow::{
        artifact_payload, escrow_payload, prepare_escrow_release, prepare_task_accept,
        prepare_task_artifact_submission, prepare_task_complete, remember_artifact_alias,
        remember_escrow_alias, remember_task_alias, task_payload,
    },
    LiveTransportKamnClient,
};
use crate::{
    AgentDid, Artifact, ArtifactId, EscrowConfig, EscrowId, Message, MessageId, SdkError,
    TaskDefinition, TaskId,
};

impl LiveTransportKamnClient {
    pub(super) fn send_via_service(&self, message: Message) -> Result<MessageId, SdkError> {
        let payload = super::routes::service_message_payload(&message);
        let auth = build_auth(
            &self.state,
            &self.config,
            &message.from,
            payload.as_str(),
            Some(MESSAGES_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.send_message(payload.as_str(), &auth)?;
        remember_message_id(&self.state, receipt.message_id.as_str())
    }

    pub(super) fn create_task_via_service(
        &self,
        task: TaskDefinition,
    ) -> Result<TaskId, SdkError> {
        let payload = task_payload(&task)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &task.creator,
            payload.as_str(),
            Some(TASKS_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.create_task(payload.as_str(), &auth)?;
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        remember_task_alias(
            &mut guard.task_aliases,
            receipt.task_id.as_str(),
            &task.creator,
        )
    }

    pub(super) fn accept_task_via_service(
        &self,
        task_id: &TaskId,
        assignee: &AgentDid,
    ) -> Result<(), SdkError> {
        let (service_task_id, sender) = {
            let mut guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_task_accept(&mut guard.task_aliases, task_id, assignee)?
        };
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            "{}",
            Some(TASKS_WRITE_SCOPE),
        )?;
        self.service_client
            .accept_task(service_task_id.as_str(), &auth)?;
        Ok(())
    }

    pub(super) fn submit_artifact_via_service(
        &self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        let (service_task_id, sender) = {
            let guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_task_artifact_submission(&guard.task_aliases, task_id)?
        };
        let payload = artifact_payload(service_task_id.as_str(), &artifact)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            payload.as_str(),
            Some(CONTENT_WRITE_SCOPE),
        )?;
        let registration = self.service_client.register_content(payload.as_str(), &auth)?;
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        remember_artifact_alias(&mut guard.artifact_ids, registration.content_id.as_str())
    }

    pub(super) fn complete_task_via_service(&self, task_id: &TaskId) -> Result<(), SdkError> {
        let (service_task_id, sender) = {
            let guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_task_complete(&guard.task_aliases, task_id)?
        };
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            "{}",
            Some(TASKS_WRITE_SCOPE),
        )?;
        self.service_client
            .complete_task(service_task_id.as_str(), &auth)?;
        Ok(())
    }

    pub(super) fn create_escrow_via_service(
        &self,
        escrow: EscrowConfig,
    ) -> Result<EscrowId, SdkError> {
        let payload = escrow_payload(&escrow)?;
        let auth = build_auth(
            &self.state,
            &self.config,
            &escrow.payer,
            payload.as_str(),
            Some(ESCROW_WRITE_SCOPE),
        )?;
        let receipt = self.service_client.fund_escrow(payload.as_str(), &auth)?;
        let mut guard = self
            .state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        remember_escrow_alias(
            &mut guard.escrow_aliases,
            receipt.escrow_id.as_str(),
            &escrow.payer,
        )
    }

    pub(super) fn release_escrow_via_service(
        &self,
        escrow_id: &EscrowId,
    ) -> Result<(), SdkError> {
        let (service_escrow_id, sender) = {
            let guard = self
                .state
                .lock()
                .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
            prepare_escrow_release(&guard.escrow_aliases, escrow_id)?
        };
        let auth = build_auth(
            &self.state,
            &self.config,
            &sender,
            "{}",
            Some(ESCROW_WRITE_SCOPE),
        )?;
        self.service_client
            .release_escrow(service_escrow_id.as_str(), &auth)?;
        Ok(())
    }
}
