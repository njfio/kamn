use super::super::*;

mod dispatch;
mod escrow_lifecycle;
mod lifecycle;
mod settlement;
mod settlement_intent;
mod tasks;

use dispatch::{
    dispatch_prerequisite_missing_error, dispatch_request_from_record, select_dispatch_assignee,
};
pub(crate) use escrow_lifecycle::EscrowLifecycleError;
pub(crate) use lifecycle::TaskLifecycleError;
pub(crate) use settlement::escrow_fund_task_id;
use settlement::{escrow_status_response, release_escrow_record};
#[cfg(test)]
pub(crate) use settlement_intent::settlement_signature_is_available;
use tasks::{next_task_id, persist_task_created_audit_export};

impl ServiceApiMessageStore {
    pub(crate) fn participant_task_projection(
        &mut self,
        task_id: &str,
        requester_did: &str,
    ) -> Result<Option<ServiceApiParticipantTaskProjection>, TaskProjectionError> {
        super::super::task_projection::participant(self, task_id, requester_did)
    }

    pub(crate) fn verifier_task_projection(
        &mut self,
        task_id: &str,
        requester_did: &str,
    ) -> Result<Option<ServiceApiVerifierTaskProjection>, TaskProjectionError> {
        super::super::task_projection::verifier(self, task_id, requester_did)
    }

    pub(crate) fn prepare_settlement_intent(
        &mut self,
        actor: &str,
        escrow_id: &str,
        idempotency_key: &str,
        prepared: &crate::service_api_endpoint::live_settlement_dispatch::PreparedLiveSettlement,
    ) -> Result<ServiceApiSettlementIntentRecord, String> {
        settlement_intent::prepare(self, actor, escrow_id, idempotency_key, prepared)
    }

    pub(crate) fn get_settlement_intent(
        &mut self,
        escrow_id: &str,
    ) -> Result<Option<ServiceApiSettlementIntentRecord>, String> {
        self.refresh_from_disk()?;
        Ok(self.snapshot.settlement_intents.get(escrow_id).cloned())
    }

    pub(crate) fn finalize_settlement_intent(
        &mut self,
        escrow_id: &str,
        settlement: &ServiceApiSettlementMetadata,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        settlement_intent::finalize(self, escrow_id, settlement)
    }

    pub(crate) fn mark_settlement_outcome_ambiguous(
        &mut self,
        escrow_id: &str,
    ) -> Result<(), String> {
        settlement_intent::mark_ambiguous(self, escrow_id)
    }

    pub(crate) fn mark_settlement_failed(
        &mut self,
        escrow_id: &str,
        error_code: &str,
    ) -> Result<(), String> {
        settlement_intent::mark_failed(self, escrow_id, error_code)
    }

    pub(crate) fn mark_settlement_expired(&mut self, escrow_id: &str) -> Result<(), String> {
        settlement_intent::mark_expired(self, escrow_id)
    }

    pub(crate) fn fund_bound_escrow(
        &mut self,
        actor: &str,
        payload: &str,
        correlation_id: &str,
    ) -> Result<ServiceApiEscrowStatusBody, EscrowLifecycleError> {
        escrow_lifecycle::fund(self, actor, payload, correlation_id)
    }

    pub(crate) fn authorize_escrow_release(
        &mut self,
        actor: &str,
        escrow_id: &str,
        payload: &str,
        correlation_id: &str,
    ) -> Result<ServiceApiEscrowStatusBody, EscrowLifecycleError> {
        escrow_lifecycle::authorize_release(self, actor, escrow_id, payload, correlation_id)
    }

    pub(crate) fn validate_escrow_release_eligibility(
        &mut self,
        actor: &str,
        escrow_id: &str,
    ) -> Result<(), EscrowLifecycleError> {
        escrow_lifecycle::validate_release_eligibility(self, actor, escrow_id)
    }
    pub(crate) fn create_bound_task(
        &mut self,
        actor_did: &str,
        payload: &str,
        correlation_id: &str,
    ) -> Result<ServiceApiTaskCreateBody, TaskLifecycleError> {
        lifecycle::create_bound_task(self, actor_did, payload, correlation_id)
    }

    pub(crate) fn transition_bound_task(
        &mut self,
        actor_did: &str,
        task_id: &str,
        action: &str,
        payload: &str,
        correlation_id: &str,
    ) -> Result<ServiceApiTaskTransitionBody, TaskLifecycleError> {
        lifecycle::transition_bound_task(self, actor_did, task_id, action, payload, correlation_id)
    }

    pub(crate) fn get_task(
        &mut self,
        task_id: &str,
    ) -> Result<Option<ServiceApiTaskGetBody>, String> {
        self.refresh_from_disk()?;
        self.dispatch_task_if_ready(task_id)?;
        let Some(record) = self.snapshot.tasks.get(task_id) else {
            return Ok(None);
        };
        Ok(Some(ServiceApiTaskGetBody {
            task_id: record.task_id.clone(),
            state: record.state.clone(),
        }))
    }

    pub(crate) fn get_escrow_status(
        &mut self,
        escrow_id: &str,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.escrows.get(escrow_id) else {
            return Ok(None);
        };
        Ok(Some(escrow_status_response(record)))
    }

    pub(crate) fn release_escrow(
        &mut self,
        escrow_id: &str,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.release_escrow_inner(escrow_id, None)
    }

    pub(crate) fn release_escrow_with_settlement_receipt_hash(
        &mut self,
        escrow_id: &str,
        settlement_receipt_hash: &str,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.release_escrow_inner(
            escrow_id,
            Some(&ServiceApiSettlementMetadata {
                settlement_receipt_hash: Some(settlement_receipt_hash.to_owned()),
                ..ServiceApiSettlementMetadata::default()
            }),
        )
    }

    fn release_escrow_inner(
        &mut self,
        escrow_id: &str,
        settlement: Option<&ServiceApiSettlementMetadata>,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.refresh_from_disk()?;
        let response = {
            let Some(record) = self.snapshot.escrows.get_mut(escrow_id) else {
                return Ok(None);
            };
            if record.state == "released" {
                return Ok(Some(escrow_status_response(record)));
            }
            release_escrow_record(record, settlement);
            escrow_status_response(record)
        };
        self.persist()?;
        Ok(Some(response))
    }

    fn dispatch_task_if_ready(&mut self, task_id: &str) -> Result<(), String> {
        let Some(record) = self.snapshot.tasks.get(task_id).cloned() else {
            return Ok(());
        };
        if record.provider_did.is_some() {
            return Ok(());
        }
        if record.state != "submitted" {
            return Ok(());
        }
        let Some(dispatch_request) = dispatch_request_from_record(&record)? else {
            return Ok(());
        };
        let assignee = select_dispatch_assignee(&self.snapshot.agents, &dispatch_request)
            .ok_or_else(|| {
                dispatch_prerequisite_missing_error(dispatch_request.task_type.as_str())
            })?;
        let Some(record) = self.snapshot.tasks.get_mut(task_id) else {
            return Ok(());
        };
        record.assignee = Some(assignee);
        record.state = "completed".to_owned();
        self.persist()
    }
}
