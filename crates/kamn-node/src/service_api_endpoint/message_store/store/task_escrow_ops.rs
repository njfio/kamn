use super::super::*;

mod dispatch;

use dispatch::{
    dispatch_prerequisite_missing_error, dispatch_request_from_record,
    parse_dispatchable_task_payload, select_dispatch_assignee, DispatchableTaskPayload,
};

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
        })
    }

    pub(crate) fn release_escrow(
        &mut self,
        escrow_id: &str,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.escrows.get_mut(escrow_id) else {
            return Ok(None);
        };
        record.state = "released".to_owned();
        self.persist()?;
        Ok(Some(ServiceApiEscrowStatusBody {
            escrow_id: escrow_id.to_owned(),
            state: "released".to_owned(),
        }))
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

fn next_task_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    next_local_task_escrow_id("task-local", payload, |candidate| {
        store.snapshot.tasks.contains_key(candidate)
    })
}

fn next_escrow_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    next_local_task_escrow_id("escrow-local", payload, |candidate| {
        store.snapshot.escrows.contains_key(candidate)
    })
}

fn next_local_task_escrow_id<F>(prefix: &str, payload: &str, exists: F) -> String
where
    F: Fn(&str) -> bool,
{
    let base = format!(
        "{prefix}-{:016x}",
        deterministic_body_tag(payload.as_bytes())
    );
    let mut candidate = base.clone();
    let mut suffix = 1_u64;
    while exists(candidate.as_str()) {
        candidate = format!("{base}-{suffix}");
        suffix = suffix.saturating_add(1);
    }
    candidate
}

fn build_task_record(
    task_id: &str,
    dispatch_metadata: Option<DispatchableTaskPayload>,
) -> ServiceApiPersistedTaskRecord {
    ServiceApiPersistedTaskRecord {
        task_id: task_id.to_owned(),
        state: "submitted".to_owned(),
        creator_did: dispatch_metadata
            .as_ref()
            .map(|metadata| metadata.creator_did.clone()),
        task_type: dispatch_metadata
            .as_ref()
            .map(|metadata| metadata.task_type.clone()),
        description: dispatch_metadata.map(|metadata| metadata.description),
        assignee: None,
    }
}

fn build_escrow_record(escrow_id: &str) -> ServiceApiPersistedEscrowRecord {
    ServiceApiPersistedEscrowRecord {
        escrow_id: escrow_id.to_owned(),
        state: "funded".to_owned(),
    }
}

fn persist_task_created_audit_export(
    store: &ServiceApiMessageStore,
    task_id: &str,
) -> Result<(), String> {
    let event = service_api_task_created_audit_event(task_id);
    persist_service_api_audit_export_event(store.audit_export_file.as_deref(), event)
}
