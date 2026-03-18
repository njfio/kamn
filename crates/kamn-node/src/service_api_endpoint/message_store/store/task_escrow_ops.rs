use super::super::*;

mod dispatch;
mod settlement;
mod tasks;

use dispatch::{
    dispatch_prerequisite_missing_error, dispatch_request_from_record,
    parse_dispatchable_task_payload, select_dispatch_assignee,
};
use settlement::{
    build_escrow_record, escrow_status_response, next_escrow_id, release_escrow_record,
};
use tasks::{build_task_record, next_task_id, persist_task_created_audit_export};

impl ServiceApiMessageStore {
    pub(crate) fn create_task(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiTaskCreateBody, String> {
        self.refresh_from_disk()?;
        let task_id = next_task_id(self, payload);
        let dispatch_metadata = parse_dispatchable_task_payload(payload)?;
        self.snapshot.tasks.insert(
            task_id.clone(),
            build_task_record(task_id.as_str(), dispatch_metadata),
        );
        self.persist()?;
        persist_task_created_audit_export(self, task_id.as_str())?;
        Ok(ServiceApiTaskCreateBody {
            task_id,
            state: "submitted".to_owned(),
        })
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

    pub(crate) fn transition_task(
        &mut self,
        task_id: &str,
        state: &str,
    ) -> Result<Option<ServiceApiTaskTransitionBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.tasks.get_mut(task_id) else {
            return Ok(None);
        };
        record.state = state.to_owned();
        self.persist()?;
        Ok(Some(ServiceApiTaskTransitionBody {
            task_id: task_id.to_owned(),
            state: state.to_owned(),
        }))
    }

    pub(crate) fn fund_escrow(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiEscrowStatusBody, String> {
        self.refresh_from_disk()?;
        let escrow_id = next_escrow_id(self, payload);
        self.snapshot
            .escrows
            .insert(escrow_id.clone(), build_escrow_record(escrow_id.as_str()));
        self.persist()?;
        Ok(ServiceApiEscrowStatusBody {
            escrow_id,
            state: "funded".to_owned(),
            settlement: ServiceApiSettlementMetadata::default(),
        })
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

    pub(crate) fn release_escrow_with_settlement_metadata(
        &mut self,
        escrow_id: &str,
        settlement: &ServiceApiSettlementMetadata,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.release_escrow_inner(escrow_id, Some(settlement))
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
