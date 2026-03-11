#![allow(missing_docs)]

use crate::{TaskLifecycle, TaskState};

/// Notice kinds emitted for task operation lifecycle activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskOperationNoticeKind {
    Submitted,
    Accepted,
    Delegated,
    Started,
    InputRequired,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

/// Canonical mutable task operation record tracked by engine state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecord {
    pub task_id: String,
    pub requester: String,
    pub assignee: Option<String>,
    pub description: String,
    pub lifecycle: TaskLifecycle,
}

/// Draft payload for submitting a task batch with dependency edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmTaskDraft {
    pub task_id: String,
    pub requester: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

/// Schema version for serialized task operation snapshots.
pub const TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// Snapshot projection for a single task operation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecordSnapshot {
    pub task_id: String,
    pub requester: String,
    pub assignee: Option<String>,
    pub description: String,
    pub lifecycle_history: Vec<TaskState>,
    pub dependencies: Vec<String>,
    pub notices: Vec<TaskOperationNoticeKind>,
}

/// Serialized snapshot for all task operation engine records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationSnapshot {
    pub schema_version: u16,
    pub tasks: Vec<TaskOperationRecordSnapshot>,
}
