use super::super::*;

impl ServiceApiMessageStore {
    pub(crate) fn create_task(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiTaskCreateBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "task-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut task_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.tasks.contains_key(task_id.as_str()) {
            task_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.tasks.insert(
            task_id.clone(),
            ServiceApiPersistedTaskRecord {
                task_id: task_id.clone(),
                state: "submitted".to_owned(),
            },
        );
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
        let base = format!(
            "escrow-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut escrow_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.escrows.contains_key(escrow_id.as_str()) {
            escrow_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.escrows.insert(
            escrow_id.clone(),
            ServiceApiPersistedEscrowRecord {
                escrow_id: escrow_id.clone(),
                state: "funded".to_owned(),
            },
        );
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
