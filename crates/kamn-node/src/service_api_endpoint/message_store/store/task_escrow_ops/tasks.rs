use super::super::super::{
    audit_export::{persist_service_api_audit_export_event, service_api_task_created_audit_event},
    ServiceApiMessageStore,
};
use crate::service_api_endpoint::payload::deterministic_body_tag;

pub(super) fn next_task_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    next_local_task_escrow_id("task-local", payload, |candidate| {
        store.snapshot.tasks.contains_key(candidate)
    })
}

pub(super) fn persist_task_created_audit_export(
    store: &ServiceApiMessageStore,
    task_id: &str,
) -> Result<(), String> {
    let event = service_api_task_created_audit_event(task_id);
    persist_service_api_audit_export_event(store.audit_export_file.as_deref(), event)
}

pub(super) fn next_local_task_escrow_id<F>(prefix: &str, payload: &str, exists: F) -> String
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
