use super::super::*;

impl ServiceApiMessageStore {
    pub(crate) fn create_task(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiTaskCreateBody, String> {
        self.refresh_from_disk()?;
        let task_id = next_task_id(self, payload);
        self.snapshot
            .tasks
            .insert(task_id.clone(), build_task_record(task_id.as_str()));
        self.persist()?;
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

fn build_task_record(task_id: &str) -> ServiceApiPersistedTaskRecord {
    ServiceApiPersistedTaskRecord {
        task_id: task_id.to_owned(),
        state: "submitted".to_owned(),
    }
}

fn build_escrow_record(escrow_id: &str) -> ServiceApiPersistedEscrowRecord {
    ServiceApiPersistedEscrowRecord {
        escrow_id: escrow_id.to_owned(),
        state: "funded".to_owned(),
    }
}
