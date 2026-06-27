#![allow(missing_docs)]

use crate::{TaskLifecycleError, TaskState};
use std::fmt;

/// Errors emitted by task operation lifecycle, dependency graph, and snapshot restore flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOperationError {
    NotFound(String),
    DuplicateTaskId(String),
    EmptySwarmTaskSet,
    DuplicateDependency {
        task_id: String,
        dependency_id: String,
    },
    UnknownDependency {
        task_id: String,
        dependency_id: String,
    },
    CyclicDependency {
        task_id: String,
    },
    DependencyNotSatisfied {
        task_id: String,
        dependency_id: String,
    },
    SnapshotVersionMismatch {
        expected: u16,
        found: u16,
    },
    SnapshotDependencyNotCompleted {
        task_id: String,
        dependency_id: String,
        dependency_state: TaskState,
    },
    InvalidSnapshot(String),
    InvalidDid(String),
    EmptyDescription,
    EmptyReason(&'static str),
    UnauthorizedActor {
        actor: String,
        required: &'static str,
    },
    Lifecycle(String),
}

impl fmt::Display for TaskOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(value) => write!(f, "task not found: {value}"),
            Self::DuplicateTaskId(value) => write!(f, "duplicate task id: {value}"),
            Self::EmptySwarmTaskSet => write!(f, "swarm task set must not be empty"),
            Self::DuplicateDependency { .. }
            | Self::UnknownDependency { .. }
            | Self::CyclicDependency { .. }
            | Self::DependencyNotSatisfied { .. } => write_dependency_error(self, f),
            Self::SnapshotVersionMismatch { .. } | Self::SnapshotDependencyNotCompleted { .. } => {
                write_snapshot_error(self, f)
            }
            Self::InvalidSnapshot(value) => write!(f, "invalid task operation snapshot: {value}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::EmptyDescription => write!(f, "task description must not be empty"),
            Self::EmptyReason(action) => write!(f, "reason must not be empty for {action}"),
            Self::UnauthorizedActor { actor, required } => write_actor_error(actor, required, f),
            Self::Lifecycle(value) => write!(f, "task lifecycle error: {value}"),
        }
    }
}

impl std::error::Error for TaskOperationError {}

/// Errors emitted by task operation snapshot-store serialization/persistence operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOperationSnapshotStoreError {
    Io(String),
    InvalidPayload(String),
    Snapshot(TaskOperationError),
}

impl fmt::Display for TaskOperationSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(value) => write!(f, "task operation snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(f, "task operation snapshot store invalid payload: {value}")
            }
            Self::Snapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for TaskOperationSnapshotStoreError {}

pub(crate) fn lifecycle_error(error: TaskLifecycleError) -> TaskOperationError {
    TaskOperationError::Lifecycle(error.to_string())
}

fn write_dependency_error(error: &TaskOperationError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        TaskOperationError::DuplicateDependency {
            task_id,
            dependency_id,
        } => {
            write!(f, "duplicate dependency {dependency_id} for task {task_id}")
        }
        TaskOperationError::UnknownDependency {
            task_id,
            dependency_id,
        } => {
            write!(f, "unknown dependency {dependency_id} for task {task_id}")
        }
        TaskOperationError::CyclicDependency { task_id } => {
            write!(f, "cyclic task dependency detected at task {task_id}")
        }
        TaskOperationError::DependencyNotSatisfied {
            task_id,
            dependency_id,
        } => write!(
            f,
            "task {task_id} cannot start before dependency {dependency_id} is completed"
        ),
        _ => write!(
            f,
            "task operation dependency formatter route mismatch: {error:?}"
        ),
    }
}

fn write_snapshot_error(error: &TaskOperationError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        TaskOperationError::SnapshotVersionMismatch { expected, found } => {
            write!(f, "snapshot schema version mismatch, expected {expected}, found {found}")
        }
        TaskOperationError::SnapshotDependencyNotCompleted {
            task_id,
            dependency_id,
            dependency_state,
        } => write!(
            f,
            "task {task_id} has dependency {dependency_id} in {dependency_state:?} during snapshot restore"
        ),
        _ => write!(f, "task operation snapshot formatter route mismatch: {error:?}"),
    }
}

fn write_actor_error(
    actor: &str,
    required: &'static str,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(f, "unauthorized actor {actor}, requires {required}")
}
