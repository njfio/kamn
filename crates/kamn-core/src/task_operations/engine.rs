use crate::{AgentDid, TaskLifecycle, TaskState, TaskTransition};
use std::collections::{BTreeMap, BTreeSet};

use super::models::{
    lifecycle_error, SwarmTaskDraft, TaskOperationError, TaskOperationNoticeKind,
    TaskOperationRecord, TaskOperationRecordSnapshot, TaskOperationSnapshot,
    TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
};

mod restore_support;
mod snapshot_projection;
mod snapshot_restore_validation;
mod submission;
mod support;
mod transition_support;
mod transitions;

/// In-memory engine for task operations, dependency gating, and lifecycle transitions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskOperationEngine {
    tasks: BTreeMap<String, TaskOperationRecord>,
    notices_by_task: BTreeMap<String, Vec<TaskOperationNoticeKind>>,
    dependencies_by_task: BTreeMap<String, BTreeSet<String>>,
}

impl TaskOperationEngine {
    /// Construct an empty task operation engine.
    pub fn new() -> Self {
        Self::default()
    }
}

pub(super) fn validate_did(value: &str) -> Result<(), TaskOperationError> {
    AgentDid::parse(value).map_err(|error| TaskOperationError::InvalidDid(error.to_string()))?;
    Ok(())
}
