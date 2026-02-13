//! Task operation workflow contracts, dependency orchestration, and snapshot persistence.

use crate::{AgentDid, TaskLifecycle, TaskLifecycleError, TaskState, TaskTransition};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Notice kinds emitted for task operation lifecycle activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskOperationNoticeKind {
    /// Task was submitted by requester.
    Submitted,
    /// Task was accepted by assignee.
    Accepted,
    /// Task assignment was delegated.
    Delegated,
    /// Assignee started task work.
    Started,
    /// Assignee requested additional input.
    InputRequired,
    /// Task was blocked due to external/internal issue.
    Blocked,
    /// Task was completed successfully.
    Completed,
    /// Task was marked failed.
    Failed,
    /// Task was cancelled by requester or assignee.
    Cancelled,
}

/// Canonical mutable task operation record tracked by engine state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecord {
    /// Unique task identifier.
    pub task_id: String,
    /// DID of requester that submitted the task.
    pub requester: String,
    /// DID of assigned worker, when assigned.
    pub assignee: Option<String>,
    /// Human-readable task description.
    pub description: String,
    /// Task lifecycle state machine and transition history.
    pub lifecycle: TaskLifecycle,
}

/// Draft payload for submitting a task batch with dependency edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmTaskDraft {
    /// Unique task identifier.
    pub task_id: String,
    /// DID of requester that submitted the task.
    pub requester: String,
    /// Human-readable task description.
    pub description: String,
    /// Dependency task identifiers that must complete before work can start.
    pub dependencies: Vec<String>,
}

/// Schema version for serialized task operation snapshots.
pub const TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// Snapshot projection for a single task operation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecordSnapshot {
    /// Unique task identifier.
    pub task_id: String,
    /// DID of requester that submitted the task.
    pub requester: String,
    /// DID of assigned worker, when assigned.
    pub assignee: Option<String>,
    /// Human-readable task description.
    pub description: String,
    /// Serialized lifecycle history in transition order.
    pub lifecycle_history: Vec<TaskState>,
    /// Serialized dependency identifiers.
    pub dependencies: Vec<String>,
    /// Serialized task operation notices.
    pub notices: Vec<TaskOperationNoticeKind>,
}

/// Serialized snapshot for all task operation engine records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationSnapshot {
    /// Snapshot schema version.
    pub schema_version: u16,
    /// Serialized task records contained in snapshot.
    pub tasks: Vec<TaskOperationRecordSnapshot>,
}

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

    /// Submit a single task with requester and description metadata.
    pub fn submit(
        &mut self,
        task_id: &str,
        requester: &str,
        description: &str,
    ) -> Result<(), TaskOperationError> {
        if self.tasks.contains_key(task_id) {
            return Err(TaskOperationError::DuplicateTaskId(task_id.to_owned()));
        }
        validate_did(requester)?;
        if description.trim().is_empty() {
            return Err(TaskOperationError::EmptyDescription);
        }

        let lifecycle = TaskLifecycle::new(task_id)
            .map_err(|error| TaskOperationError::Lifecycle(error.to_string()))?;
        self.tasks.insert(
            task_id.to_owned(),
            TaskOperationRecord {
                task_id: task_id.to_owned(),
                requester: requester.to_owned(),
                assignee: None,
                description: description.to_owned(),
                lifecycle,
            },
        );
        self.dependencies_by_task
            .entry(task_id.to_owned())
            .or_default();
        self.push_notice(task_id, TaskOperationNoticeKind::Submitted);
        Ok(())
    }

    /// Submit a batch of tasks and validate dependency graph integrity.
    pub fn submit_swarm_tasks(
        &mut self,
        drafts: Vec<SwarmTaskDraft>,
    ) -> Result<(), TaskOperationError> {
        if drafts.is_empty() {
            return Err(TaskOperationError::EmptySwarmTaskSet);
        }

        let mut graph = BTreeMap::new();
        let mut draft_ids = BTreeSet::new();
        for draft in &drafts {
            if self.tasks.contains_key(&draft.task_id) || !draft_ids.insert(draft.task_id.clone()) {
                return Err(TaskOperationError::DuplicateTaskId(draft.task_id.clone()));
            }
            validate_did(&draft.requester)?;
            if draft.description.trim().is_empty() {
                return Err(TaskOperationError::EmptyDescription);
            }

            let mut unique_dependencies = BTreeSet::new();
            for dependency_id in &draft.dependencies {
                if !unique_dependencies.insert(dependency_id.clone()) {
                    return Err(TaskOperationError::DuplicateDependency {
                        task_id: draft.task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }
            graph.insert(draft.task_id.clone(), unique_dependencies);
        }

        for (task_id, dependencies) in &graph {
            for dependency_id in dependencies {
                if !draft_ids.contains(dependency_id) && !self.tasks.contains_key(dependency_id) {
                    return Err(TaskOperationError::UnknownDependency {
                        task_id: task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }
        }

        let cycle_task_id = detect_cycle_task_id(&graph);
        if let Some(task_id) = cycle_task_id {
            return Err(TaskOperationError::CyclicDependency { task_id });
        }

        let mut dependencies_by_task = BTreeMap::new();
        for draft in &drafts {
            dependencies_by_task.insert(
                draft.task_id.clone(),
                graph.get(&draft.task_id).cloned().unwrap_or_default(),
            );
        }

        for draft in drafts {
            self.submit(&draft.task_id, &draft.requester, &draft.description)?;
            let dependencies = dependencies_by_task
                .remove(&draft.task_id)
                .unwrap_or_default();
            self.dependencies_by_task
                .insert(draft.task_id, dependencies);
        }
        Ok(())
    }

    /// Accept a task as assignee and transition lifecycle to accepted state.
    pub fn accept(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        let record = self.task_mut(task_id)?;

        if let Some(current) = record.assignee.as_deref() {
            if current != actor {
                return Err(TaskOperationError::UnauthorizedActor {
                    actor: actor.to_owned(),
                    required: "unassigned_or_current_assignee",
                });
            }
        }

        record
            .lifecycle
            .transition(TaskTransition::Accept)
            .map_err(lifecycle_error)?;
        record.assignee = Some(actor.to_owned());
        self.push_notice(task_id, TaskOperationNoticeKind::Accepted);
        Ok(())
    }

    /// Delegate a task from current assignee to a new assignee.
    pub fn delegate(
        &mut self,
        task_id: &str,
        actor: &str,
        delegatee: &str,
    ) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        validate_did(delegatee)?;
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;

        record
            .lifecycle
            .transition(TaskTransition::Delegate)
            .map_err(lifecycle_error)?;
        record.assignee = Some(delegatee.to_owned());
        self.push_notice(task_id, TaskOperationNoticeKind::Delegated);
        Ok(())
    }

    /// Start task work after dependency-satisfaction checks pass.
    pub fn start_work(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        if let Some(dependency_id) = self.unsatisfied_dependency(task_id)? {
            return Err(TaskOperationError::DependencyNotSatisfied {
                task_id: task_id.to_owned(),
                dependency_id,
            });
        }
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        record
            .lifecycle
            .transition(TaskTransition::StartWork)
            .map_err(lifecycle_error)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Started);
        Ok(())
    }

    /// Mark task as blocked with a non-empty reason.
    pub fn block(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        if reason.trim().is_empty() {
            return Err(TaskOperationError::EmptyReason("block"));
        }
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        record
            .lifecycle
            .transition(TaskTransition::Block)
            .map_err(lifecycle_error)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Blocked);
        Ok(())
    }

    /// Move task into input-required state with a non-empty reason.
    pub fn request_input(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        if reason.trim().is_empty() {
            return Err(TaskOperationError::EmptyReason("request_input"));
        }
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        record
            .lifecycle
            .transition(TaskTransition::RequestInput)
            .map_err(lifecycle_error)?;
        self.push_notice(task_id, TaskOperationNoticeKind::InputRequired);
        Ok(())
    }

    /// Complete task as assignee.
    pub fn complete(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        record
            .lifecycle
            .transition(TaskTransition::Complete)
            .map_err(lifecycle_error)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Completed);
        Ok(())
    }

    /// Mark task as failed with a non-empty reason.
    pub fn fail(
        &mut self,
        task_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        if reason.trim().is_empty() {
            return Err(TaskOperationError::EmptyReason("fail"));
        }
        let record = self.task_mut(task_id)?;
        Self::require_assignee(record, actor)?;
        record
            .lifecycle
            .transition(TaskTransition::Fail)
            .map_err(lifecycle_error)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Failed);
        Ok(())
    }

    /// Cancel a task as requester or current assignee.
    pub fn cancel(&mut self, task_id: &str, actor: &str) -> Result<(), TaskOperationError> {
        validate_did(actor)?;
        let record = self.task_mut(task_id)?;
        let is_requester = record.requester == actor;
        let is_assignee = record.assignee.as_deref() == Some(actor);
        if !is_requester && !is_assignee {
            return Err(TaskOperationError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "requester_or_assignee",
            });
        }

        record
            .lifecycle
            .transition(TaskTransition::Cancel)
            .map_err(lifecycle_error)?;
        self.push_notice(task_id, TaskOperationNoticeKind::Cancelled);
        Ok(())
    }

    /// Return immutable task record by identifier.
    pub fn task(&self, task_id: &str) -> Result<&TaskOperationRecord, TaskOperationError> {
        self.tasks
            .get(task_id)
            .ok_or_else(|| TaskOperationError::NotFound(task_id.to_owned()))
    }

    /// Return task notice history in insertion order for the given task.
    pub fn notices(&self, task_id: &str) -> Vec<TaskOperationNoticeKind> {
        self.notices_by_task
            .get(task_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Return task identifiers that are accepted/delegated and dependency-ready.
    pub fn ready_tasks(&self) -> Vec<String> {
        self.tasks
            .iter()
            .filter_map(|(task_id, record)| {
                let state = record.lifecycle.state();
                if state != TaskState::Accepted && state != TaskState::Delegated {
                    return None;
                }
                if self
                    .unsatisfied_dependency(task_id)
                    .ok()
                    .flatten()
                    .is_some()
                {
                    return None;
                }
                Some(task_id.clone())
            })
            .collect()
    }

    /// Export full engine state into serializable snapshot form.
    pub fn export_snapshot(&self) -> TaskOperationSnapshot {
        let tasks = self
            .tasks
            .iter()
            .map(|(task_id, record)| TaskOperationRecordSnapshot {
                task_id: task_id.clone(),
                requester: record.requester.clone(),
                assignee: record.assignee.clone(),
                description: record.description.clone(),
                lifecycle_history: record.lifecycle.history(),
                dependencies: self
                    .dependencies_by_task
                    .get(task_id)
                    .map(|values| values.iter().cloned().collect())
                    .unwrap_or_default(),
                notices: self.notices(task_id),
            })
            .collect();
        TaskOperationSnapshot {
            schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
            tasks,
        }
    }

    /// Restore engine state from snapshot after schema, graph, and lifecycle validation.
    pub fn restore_snapshot(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationError> {
        if snapshot.schema_version != TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION {
            return Err(TaskOperationError::SnapshotVersionMismatch {
                expected: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        let mut restored_tasks = BTreeMap::new();
        let mut restored_notices = BTreeMap::new();
        let mut restored_dependencies = BTreeMap::new();

        for task in snapshot.tasks {
            if restored_tasks.contains_key(&task.task_id) {
                return Err(TaskOperationError::DuplicateTaskId(task.task_id));
            }
            validate_did(&task.requester)?;
            if let Some(assignee) = &task.assignee {
                validate_did(assignee)?;
            }
            if task.description.trim().is_empty() {
                return Err(TaskOperationError::InvalidSnapshot(format!(
                    "task {} has empty description",
                    task.task_id
                )));
            }
            let lifecycle = TaskLifecycle::restore(&task.task_id, task.lifecycle_history.clone())
                .map_err(|error| {
                TaskOperationError::InvalidSnapshot(format!(
                    "task {} has invalid lifecycle history: {error}",
                    task.task_id
                ))
            })?;

            let mut dependency_set = BTreeSet::new();
            for dependency_id in &task.dependencies {
                if !dependency_set.insert(dependency_id.clone()) {
                    return Err(TaskOperationError::DuplicateDependency {
                        task_id: task.task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }

            restored_notices.insert(task.task_id.clone(), task.notices);
            restored_dependencies.insert(task.task_id.clone(), dependency_set);
            restored_tasks.insert(
                task.task_id.clone(),
                TaskOperationRecord {
                    task_id: task.task_id,
                    requester: task.requester,
                    assignee: task.assignee,
                    description: task.description,
                    lifecycle,
                },
            );
        }

        for (task_id, dependencies) in &restored_dependencies {
            for dependency_id in dependencies {
                if !restored_tasks.contains_key(dependency_id) {
                    return Err(TaskOperationError::UnknownDependency {
                        task_id: task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    });
                }
            }
        }

        if let Some(task_id) = detect_cycle_task_id(&restored_dependencies) {
            return Err(TaskOperationError::CyclicDependency { task_id });
        }

        for (task_id, dependencies) in &restored_dependencies {
            let task_state = restored_tasks
                .get(task_id)
                .map(|task| task.lifecycle.state())
                .ok_or_else(|| TaskOperationError::NotFound(task_id.clone()))?;
            if !requires_completed_dependencies(task_state) {
                continue;
            }
            for dependency_id in dependencies {
                let dependency_state = restored_tasks
                    .get(dependency_id)
                    .map(|task| task.lifecycle.state())
                    .ok_or_else(|| TaskOperationError::UnknownDependency {
                        task_id: task_id.clone(),
                        dependency_id: dependency_id.clone(),
                    })?;
                if dependency_state != TaskState::Completed {
                    return Err(TaskOperationError::SnapshotDependencyNotCompleted {
                        task_id: task_id.clone(),
                        dependency_id: dependency_id.clone(),
                        dependency_state,
                    });
                }
            }
        }

        self.tasks = restored_tasks;
        self.notices_by_task = restored_notices;
        self.dependencies_by_task = restored_dependencies;
        Ok(())
    }

    fn task_mut(&mut self, task_id: &str) -> Result<&mut TaskOperationRecord, TaskOperationError> {
        self.tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskOperationError::NotFound(task_id.to_owned()))
    }

    fn require_assignee(
        record: &TaskOperationRecord,
        actor: &str,
    ) -> Result<(), TaskOperationError> {
        if record.assignee.as_deref() != Some(actor) {
            return Err(TaskOperationError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "assignee",
            });
        }
        Ok(())
    }

    fn push_notice(&mut self, task_id: &str, notice: TaskOperationNoticeKind) {
        self.notices_by_task
            .entry(task_id.to_owned())
            .or_default()
            .push(notice);
    }

    fn unsatisfied_dependency(&self, task_id: &str) -> Result<Option<String>, TaskOperationError> {
        if !self.tasks.contains_key(task_id) {
            return Err(TaskOperationError::NotFound(task_id.to_owned()));
        }
        let dependencies = match self.dependencies_by_task.get(task_id) {
            Some(value) => value,
            None => return Ok(None),
        };
        for dependency_id in dependencies {
            let dependency = self.tasks.get(dependency_id).ok_or_else(|| {
                TaskOperationError::UnknownDependency {
                    task_id: task_id.to_owned(),
                    dependency_id: dependency_id.clone(),
                }
            })?;
            if dependency.lifecycle.state() != TaskState::Completed {
                return Ok(Some(dependency_id.clone()));
            }
        }
        Ok(None)
    }
}

/// Errors emitted by task operation lifecycle, dependency graph, and snapshot restore flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOperationError {
    /// Task identifier was not found.
    NotFound(String),
    /// Task identifier already exists.
    DuplicateTaskId(String),
    /// Swarm submission provided no tasks.
    EmptySwarmTaskSet,
    /// Duplicate dependency edge was declared for a task.
    DuplicateDependency {
        /// Task identifier containing duplicate dependency.
        task_id: String,
        /// Duplicate dependency identifier.
        dependency_id: String,
    },
    /// Dependency references an unknown task identifier.
    UnknownDependency {
        /// Task identifier referencing unknown dependency.
        task_id: String,
        /// Unknown dependency identifier.
        dependency_id: String,
    },
    /// Dependency graph contains a cycle.
    CyclicDependency {
        /// Task identifier where cycle detection terminated.
        task_id: String,
    },
    /// Task cannot proceed because dependency is not completed.
    DependencyNotSatisfied {
        /// Task identifier blocked by dependency.
        task_id: String,
        /// Unsatisfied dependency identifier.
        dependency_id: String,
    },
    /// Snapshot schema version does not match expected contract.
    SnapshotVersionMismatch {
        /// Expected snapshot schema version.
        expected: u16,
        /// Snapshot schema version found in payload.
        found: u16,
    },
    /// Snapshot restore detected dependency that is not completed for an in-progress terminal task.
    SnapshotDependencyNotCompleted {
        /// Task identifier blocked by dependency state.
        task_id: String,
        /// Dependency identifier in invalid state for restore.
        dependency_id: String,
        /// Dependency state observed in snapshot payload.
        dependency_state: TaskState,
    },
    /// Snapshot payload failed semantic validation.
    InvalidSnapshot(String),
    /// DID parse/validation failed.
    InvalidDid(String),
    /// Task description is empty.
    EmptyDescription,
    /// Required reason field is empty for a transition action.
    EmptyReason(&'static str),
    /// Actor is not authorized for requested operation.
    UnauthorizedActor {
        /// Actor DID attempting operation.
        actor: String,
        /// Required actor policy for operation.
        required: &'static str,
    },
    /// Wrapped lifecycle transition error.
    Lifecycle(String),
}

impl fmt::Display for TaskOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(value) => write!(f, "task not found: {value}"),
            Self::DuplicateTaskId(value) => write!(f, "duplicate task id: {value}"),
            Self::EmptySwarmTaskSet => write!(f, "swarm task set must not be empty"),
            Self::DuplicateDependency {
                task_id,
                dependency_id,
            } => write!(f, "duplicate dependency {dependency_id} for task {task_id}"),
            Self::UnknownDependency {
                task_id,
                dependency_id,
            } => write!(f, "unknown dependency {dependency_id} for task {task_id}"),
            Self::CyclicDependency { task_id } => {
                write!(f, "cyclic task dependency detected at task {task_id}")
            }
            Self::DependencyNotSatisfied {
                task_id,
                dependency_id,
            } => write!(
                f,
                "task {task_id} cannot start before dependency {dependency_id} is completed"
            ),
            Self::SnapshotVersionMismatch { expected, found } => write!(
                f,
                "snapshot schema version mismatch, expected {expected}, found {found}"
            ),
            Self::SnapshotDependencyNotCompleted {
                task_id,
                dependency_id,
                dependency_state,
            } => write!(
                f,
                "task {task_id} has dependency {dependency_id} in {dependency_state:?} during snapshot restore"
            ),
            Self::InvalidSnapshot(value) => write!(f, "invalid task operation snapshot: {value}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::EmptyDescription => write!(f, "task description must not be empty"),
            Self::EmptyReason(action) => write!(f, "reason must not be empty for {action}"),
            Self::UnauthorizedActor { actor, required } => {
                write!(f, "unauthorized actor {actor}, requires {required}")
            }
            Self::Lifecycle(value) => write!(f, "task lifecycle error: {value}"),
        }
    }
}

impl std::error::Error for TaskOperationError {}

/// Errors emitted by task operation snapshot-store serialization/persistence operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOperationSnapshotStoreError {
    /// Underlying filesystem I/O operation failed.
    Io(String),
    /// Serialized payload is malformed or violates delimiter contract.
    InvalidPayload(String),
    /// Wrapped snapshot validation error from task operation engine.
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

/// Snapshot persistence abstraction for task operation state.
pub trait TaskOperationSnapshotStore {
    /// Persist latest snapshot payload.
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError>;
    /// Read latest snapshot payload if present.
    fn read_latest(&self)
        -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError>;
}

/// In-memory snapshot store implementation for tests and ephemeral runtime paths.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryTaskOperationSnapshotStore {
    latest: Option<TaskOperationSnapshot>,
}

impl TaskOperationSnapshotStore for InMemoryTaskOperationSnapshotStore {
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(
        &self,
    ) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}

/// Filesystem-backed snapshot store implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTaskOperationSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

/// Recovery outcome for filesystem snapshot repair flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecoveryResult {
    /// Latest valid snapshot after recovery path.
    pub latest: Option<TaskOperationSnapshot>,
    /// Whether recovery rewrote corrupted payload to repaired baseline.
    pub repaired: bool,
    /// Deterministic recovery reason code.
    pub reason_code: &'static str,
}

impl TaskOperationRecoveryResult {
    /// Returns the deterministic recovery reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl FileTaskOperationSnapshotStore {
    /// Construct filesystem snapshot store with target path.
    pub fn new(path: PathBuf) -> Result<Self, TaskOperationSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = task_operation_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    /// Attempt to read latest snapshot; if corrupt, repair file and return empty state.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<TaskOperationRecoveryResult, TaskOperationSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(TaskOperationRecoveryResult {
                latest: None,
                repaired: false,
                reason_code: "task_operation_snapshot_recovery_empty",
            });
        }

        match self.read_latest() {
            Ok(snapshot) => Ok(TaskOperationRecoveryResult {
                latest: snapshot,
                repaired: false,
                reason_code: "task_operation_snapshot_recovery_clean",
            }),
            Err(TaskOperationSnapshotStoreError::InvalidPayload(value))
                if value.starts_with(TASK_OPERATION_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX) =>
            {
                Err(TaskOperationSnapshotStoreError::InvalidPayload(value))
            }
            Err(TaskOperationSnapshotStoreError::InvalidPayload(_))
            | Err(TaskOperationSnapshotStoreError::Snapshot(_)) => {
                fs::write(&self.path, "")
                    .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
                fs::write(&self.journal_path, "")
                    .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
                Ok(TaskOperationRecoveryResult {
                    latest: None,
                    repaired: true,
                    reason_code: "task_operation_snapshot_recovery_repaired_corrupt_payload",
                })
            }
            Err(error) => Err(error),
        }
    }
}

impl TaskOperationSnapshotStore for FileTaskOperationSnapshotStore {
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError> {
        let mut verifier = TaskOperationEngine::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(TaskOperationSnapshotStoreError::Snapshot)?;
        let payload = serialize_task_operation_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
        append_task_operation_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(
        &self,
    ) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
        let snapshot_payload = read_task_operation_snapshot_file(&self.path)?;
        let journal_snapshot = replay_task_operation_snapshot_journal(&self.journal_path)?;
        Ok(journal_snapshot.or(snapshot_payload))
    }
}

const TASK_OPERATION_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str =
    "task_operation_snapshot_journal_corrupt_tail";

fn task_operation_snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

fn read_task_operation_snapshot_file(
    path: &Path,
) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
    if !path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(path)
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok(None);
    }
    let snapshot = parse_task_operation_snapshot_payload(&payload)?;
    let mut verifier = TaskOperationEngine::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(TaskOperationSnapshotStoreError::Snapshot)?;
    Ok(Some(snapshot))
}

fn append_task_operation_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), TaskOperationSnapshotStoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path)
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
    let record = format!("entry|1|{}\n", encode_journal_hex(payload.as_bytes()));
    file.write_all(record.as_bytes())
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))
}

fn replay_task_operation_snapshot_journal(
    journal_path: &Path,
) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
    if !journal_path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(journal_path)
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
    let mut latest = None;

    for (index, line) in payload.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let payload_hex = parse_task_operation_snapshot_journal_record(trimmed, index + 1)?;
        let payload_bytes = decode_journal_hex(payload_hex)
            .ok_or_else(|| task_operation_snapshot_journal_corrupt_tail(index + 1))?;
        let payload = String::from_utf8(payload_bytes)
            .map_err(|_| task_operation_snapshot_journal_corrupt_tail(index + 1))?;
        let snapshot = parse_task_operation_snapshot_payload(&payload)
            .map_err(|_| task_operation_snapshot_journal_corrupt_tail(index + 1))?;
        let mut verifier = TaskOperationEngine::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(|_| task_operation_snapshot_journal_corrupt_tail(index + 1))?;
        latest = Some(snapshot);
    }

    Ok(latest)
}

fn parse_task_operation_snapshot_journal_record(
    line: &str,
    index: usize,
) -> Result<&str, TaskOperationSnapshotStoreError> {
    let mut parts = line.split('|');
    let Some(prefix) = parts.next() else {
        return Err(task_operation_snapshot_journal_corrupt_tail(index));
    };
    let Some(version) = parts.next() else {
        return Err(task_operation_snapshot_journal_corrupt_tail(index));
    };
    let Some(payload_hex) = parts.next() else {
        return Err(task_operation_snapshot_journal_corrupt_tail(index));
    };
    if prefix != "entry" || version != "1" || payload_hex.is_empty() || parts.next().is_some() {
        return Err(task_operation_snapshot_journal_corrupt_tail(index));
    }
    Ok(payload_hex)
}

fn task_operation_snapshot_journal_corrupt_tail(index: usize) -> TaskOperationSnapshotStoreError {
    TaskOperationSnapshotStoreError::InvalidPayload(format!(
        "{TASK_OPERATION_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}

fn encode_journal_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_journal_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0usize;
    while index < bytes.len() {
        let high = decode_journal_nibble(bytes[index])?;
        let low = decode_journal_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Some(decoded)
}

fn decode_journal_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn validate_did(value: &str) -> Result<(), TaskOperationError> {
    AgentDid::parse(value).map_err(|error| TaskOperationError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn lifecycle_error(error: TaskLifecycleError) -> TaskOperationError {
    TaskOperationError::Lifecycle(error.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Visited,
}

fn detect_cycle_task_id(graph: &BTreeMap<String, BTreeSet<String>>) -> Option<String> {
    fn dfs(
        task_id: &str,
        graph: &BTreeMap<String, BTreeSet<String>>,
        states: &mut BTreeMap<String, VisitState>,
    ) -> Option<String> {
        states.insert(task_id.to_owned(), VisitState::Visiting);
        if let Some(dependencies) = graph.get(task_id) {
            for dependency_id in dependencies {
                if !graph.contains_key(dependency_id) {
                    continue;
                }
                match states.get(dependency_id) {
                    Some(VisitState::Visiting) => return Some(dependency_id.clone()),
                    Some(VisitState::Visited) => continue,
                    None => {
                        if let Some(task_id) = dfs(dependency_id, graph, states) {
                            return Some(task_id);
                        }
                    }
                }
            }
        }
        states.insert(task_id.to_owned(), VisitState::Visited);
        None
    }

    let mut states = BTreeMap::new();
    for task_id in graph.keys() {
        if states.contains_key(task_id) {
            continue;
        }
        if let Some(cycle_task_id) = dfs(task_id, graph, &mut states) {
            return Some(cycle_task_id);
        }
    }
    None
}

fn requires_completed_dependencies(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::InProgress
            | TaskState::InputRequired
            | TaskState::Blocked
            | TaskState::Completed
            | TaskState::Failed
    )
}

fn task_state_code(state: TaskState) -> &'static str {
    match state {
        TaskState::Submitted => "0",
        TaskState::Accepted => "1",
        TaskState::Delegated => "2",
        TaskState::InProgress => "3",
        TaskState::InputRequired => "4",
        TaskState::Blocked => "5",
        TaskState::Completed => "6",
        TaskState::Failed => "7",
        TaskState::Cancelled => "8",
    }
}

fn parse_task_state_code(raw: &str) -> Option<TaskState> {
    match raw {
        "0" => Some(TaskState::Submitted),
        "1" => Some(TaskState::Accepted),
        "2" => Some(TaskState::Delegated),
        "3" => Some(TaskState::InProgress),
        "4" => Some(TaskState::InputRequired),
        "5" => Some(TaskState::Blocked),
        "6" => Some(TaskState::Completed),
        "7" => Some(TaskState::Failed),
        "8" => Some(TaskState::Cancelled),
        _ => None,
    }
}

fn task_notice_code(notice: TaskOperationNoticeKind) -> &'static str {
    match notice {
        TaskOperationNoticeKind::Submitted => "0",
        TaskOperationNoticeKind::Accepted => "1",
        TaskOperationNoticeKind::Delegated => "2",
        TaskOperationNoticeKind::Started => "3",
        TaskOperationNoticeKind::InputRequired => "4",
        TaskOperationNoticeKind::Blocked => "5",
        TaskOperationNoticeKind::Completed => "6",
        TaskOperationNoticeKind::Failed => "7",
        TaskOperationNoticeKind::Cancelled => "8",
    }
}

fn parse_task_notice_code(raw: &str) -> Option<TaskOperationNoticeKind> {
    match raw {
        "0" => Some(TaskOperationNoticeKind::Submitted),
        "1" => Some(TaskOperationNoticeKind::Accepted),
        "2" => Some(TaskOperationNoticeKind::Delegated),
        "3" => Some(TaskOperationNoticeKind::Started),
        "4" => Some(TaskOperationNoticeKind::InputRequired),
        "5" => Some(TaskOperationNoticeKind::Blocked),
        "6" => Some(TaskOperationNoticeKind::Completed),
        "7" => Some(TaskOperationNoticeKind::Failed),
        "8" => Some(TaskOperationNoticeKind::Cancelled),
        _ => None,
    }
}

fn ensure_snapshot_token(
    value: &str,
    field: &str,
    allow_comma: bool,
) -> Result<(), TaskOperationSnapshotStoreError> {
    let has_comma = !allow_comma && value.contains(',');
    if value.contains('|') || value.contains('\n') || value.contains('\r') || has_comma {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

fn serialize_task_operation_snapshot(
    snapshot: &TaskOperationSnapshot,
) -> Result<String, TaskOperationSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for task in &snapshot.tasks {
        ensure_snapshot_token(&task.task_id, "task_id", false)?;
        ensure_snapshot_token(&task.requester, "requester", false)?;
        if let Some(assignee) = &task.assignee {
            ensure_snapshot_token(assignee, "assignee", false)?;
        }
        ensure_snapshot_token(&task.description, "description", true)?;
        for dependency in &task.dependencies {
            ensure_snapshot_token(dependency, "dependency", false)?;
        }

        let assignee = task.assignee.clone().unwrap_or_default();
        let history = task
            .lifecycle_history
            .iter()
            .map(|state| task_state_code(*state))
            .collect::<Vec<_>>()
            .join(",");
        let dependencies = task.dependencies.join(",");
        let notices = task
            .notices
            .iter()
            .map(|notice| task_notice_code(*notice))
            .collect::<Vec<_>>()
            .join(",");
        payload.push_str(&format!(
            "task|{}|{}|{}|{}|{}|{}|{}\n",
            task.task_id,
            task.requester,
            assignee,
            task.description,
            history,
            dependencies,
            notices
        ));
    }
    Ok(payload)
}

fn parse_task_operation_snapshot_payload(
    payload: &str,
) -> Result<TaskOperationSnapshot, TaskOperationSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let Some(schema_line) = lines.next() else {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(
            "missing schema line".to_owned(),
        ));
    };

    let mut schema_parts = schema_line.split('|');
    let Some(schema_prefix) = schema_parts.next() else {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    let Some(schema_version_raw) = schema_parts.next() else {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    if schema_prefix != "schema" || schema_parts.next().is_some() {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    }
    let schema_version = schema_version_raw
        .parse::<u16>()
        .map_err(|_| TaskOperationSnapshotStoreError::InvalidPayload(schema_line.to_owned()))?;

    let mut tasks = Vec::new();
    for line in lines {
        let mut parts = line.split('|');
        let Some(prefix) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        if prefix != "task" {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        }
        let Some(task_id) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(requester) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(assignee_raw) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(description) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(history_raw) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(dependencies_raw) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(notices_raw) = parts.next() else {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        if parts.next().is_some() {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        }

        let lifecycle_history = if history_raw.is_empty() {
            Vec::new()
        } else {
            history_raw
                .split(',')
                .map(|raw| {
                    parse_task_state_code(raw).ok_or_else(|| {
                        TaskOperationSnapshotStoreError::InvalidPayload(line.to_owned())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let dependencies = if dependencies_raw.is_empty() {
            Vec::new()
        } else {
            dependencies_raw
                .split(',')
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        };
        let notices = if notices_raw.is_empty() {
            Vec::new()
        } else {
            notices_raw
                .split(',')
                .map(|raw| {
                    parse_task_notice_code(raw).ok_or_else(|| {
                        TaskOperationSnapshotStoreError::InvalidPayload(line.to_owned())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        tasks.push(TaskOperationRecordSnapshot {
            task_id: task_id.to_owned(),
            requester: requester.to_owned(),
            assignee: if assignee_raw.is_empty() {
                None
            } else {
                Some(assignee_raw.to_owned())
            },
            description: description.to_owned(),
            lifecycle_history,
            dependencies,
            notices,
        });
    }

    Ok(TaskOperationSnapshot {
        schema_version,
        tasks,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        serialize_task_operation_snapshot, FileTaskOperationSnapshotStore, SwarmTaskDraft,
        TaskOperationEngine, TaskOperationError, TaskOperationSnapshotStore,
        TaskOperationSnapshotStoreError,
    };
    use crate::TaskState;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn submit_rejects_duplicate_task_id() {
        let mut engine = TaskOperationEngine::new();
        assert!(engine
            .submit("task-1", "kamn:did:agent:req-1", "desc")
            .is_ok());
        assert_eq!(
            engine.submit("task-1", "kamn:did:agent:req-1", "desc"),
            Err(TaskOperationError::DuplicateTaskId("task-1".to_owned()))
        );
    }

    #[test]
    fn cancel_allowed_for_assignee() {
        let mut engine = TaskOperationEngine::new();
        assert!(engine
            .submit("task-2", "kamn:did:agent:req-1", "desc")
            .is_ok());
        assert!(engine.accept("task-2", "kamn:did:agent:worker-1").is_ok());
        assert!(engine.cancel("task-2", "kamn:did:agent:worker-1").is_ok());
        let state = match engine.task("task-2") {
            Ok(task) => task.lifecycle.state(),
            Err(error) => panic!("task lookup failed: {error}"),
        };
        assert_eq!(state, TaskState::Cancelled);
    }

    #[test]
    fn submit_swarm_rejects_cyclic_dependencies() {
        let mut engine = TaskOperationEngine::new();
        assert!(matches!(
            engine.submit_swarm_tasks(vec![
                SwarmTaskDraft {
                    task_id: "cycle-a".to_owned(),
                    requester: "kamn:did:agent:req-1".to_owned(),
                    description: "a".to_owned(),
                    dependencies: vec!["cycle-b".to_owned()],
                },
                SwarmTaskDraft {
                    task_id: "cycle-b".to_owned(),
                    requester: "kamn:did:agent:req-1".to_owned(),
                    description: "b".to_owned(),
                    dependencies: vec!["cycle-a".to_owned()],
                },
            ]),
            Err(TaskOperationError::CyclicDependency { .. })
        ));
    }

    #[test]
    fn start_work_rejects_unsatisfied_dependency() {
        let mut engine = TaskOperationEngine::new();
        engine
            .submit_swarm_tasks(vec![
                SwarmTaskDraft {
                    task_id: "dep-root".to_owned(),
                    requester: "kamn:did:agent:req-1".to_owned(),
                    description: "root".to_owned(),
                    dependencies: vec![],
                },
                SwarmTaskDraft {
                    task_id: "dep-child".to_owned(),
                    requester: "kamn:did:agent:req-1".to_owned(),
                    description: "child".to_owned(),
                    dependencies: vec!["dep-root".to_owned()],
                },
            ])
            .expect("swarm submission should succeed");
        engine
            .accept("dep-root", "kamn:did:agent:worker-1")
            .expect("root accept should succeed");
        engine
            .accept("dep-child", "kamn:did:agent:worker-1")
            .expect("child accept should succeed");

        assert_eq!(
            engine.start_work("dep-child", "kamn:did:agent:worker-1"),
            Err(TaskOperationError::DependencyNotSatisfied {
                task_id: "dep-child".to_owned(),
                dependency_id: "dep-root".to_owned(),
            })
        );
    }

    #[test]
    fn integration_file_task_operation_snapshot_store_roundtrips_snapshot() {
        let path = temp_task_operation_snapshot_path("roundtrip");
        let journal_path = temp_task_operation_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);

        let mut engine = TaskOperationEngine::new();
        engine
            .submit(
                "task-store-1",
                "kamn:did:agent:requester-1",
                "Store snapshot flow",
            )
            .expect("submit should pass");
        engine
            .accept("task-store-1", "kamn:did:agent:worker-1")
            .expect("accept should pass");

        let snapshot = engine.export_snapshot();
        let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        file_store
            .write(snapshot.clone())
            .expect("write should pass");
        assert_eq!(
            file_store.read_latest().expect("read should pass"),
            Some(snapshot)
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn integration_file_task_operation_snapshot_store_replays_journal_when_snapshot_is_stale() {
        let path = temp_task_operation_snapshot_path("journal-replay");
        let journal_path = temp_task_operation_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);

        let mut engine = TaskOperationEngine::new();
        engine
            .submit(
                "task-journal-1",
                "kamn:did:agent:requester-1",
                "first snapshot",
            )
            .expect("first submit should pass");
        let first_snapshot = engine.export_snapshot();

        let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        file_store
            .write(first_snapshot.clone())
            .expect("first write should pass");

        engine
            .submit(
                "task-journal-2",
                "kamn:did:agent:requester-1",
                "second snapshot",
            )
            .expect("second submit should pass");
        let second_snapshot = engine.export_snapshot();
        file_store
            .write(second_snapshot.clone())
            .expect("second write should pass");

        let stale_payload =
            serialize_task_operation_snapshot(&first_snapshot).expect("serialize should pass");
        assert!(fs::write(&path, stale_payload).is_ok());
        assert_eq!(
            file_store.read_latest().expect("journal replay should win"),
            Some(second_snapshot)
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn regression_file_task_operation_snapshot_store_rejects_malformed_payload() {
        // Regression: #617
        let path = temp_task_operation_snapshot_path("malformed");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "schema|1\ntask|broken\n").is_ok());

        let file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        assert_eq!(
            file_store.read_latest(),
            Err(TaskOperationSnapshotStoreError::InvalidPayload(
                "task|broken".to_owned()
            ))
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_task_operation_snapshot_store_recovery_repairs_corrupt_payload() {
        let path = temp_task_operation_snapshot_path("recover");
        let journal_path = temp_task_operation_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);
        assert!(fs::write(&path, "schema|1\ntask|broken\n").is_ok());

        let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        let recovery = file_store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert!(recovery.latest.is_none());
        assert!(recovery.repaired);
        assert_eq!(
            fs::read_to_string(&path).expect("repaired file should be readable"),
            ""
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn regression_file_task_operation_snapshot_store_rejects_corrupt_journal_tail() {
        // Regression: #2690
        let path = temp_task_operation_snapshot_path("corrupt-journal-tail");
        let journal_path = temp_task_operation_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);

        let mut engine = TaskOperationEngine::new();
        engine
            .submit("task-tail", "kamn:did:agent:requester-1", "tail payload")
            .expect("submit should pass");
        let snapshot = engine.export_snapshot();

        let mut file_store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        file_store.write(snapshot).expect("write should pass");

        let mut journal = OpenOptions::new()
            .append(true)
            .open(&journal_path)
            .expect("journal should exist");
        assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
        assert_eq!(
            file_store.recover_latest_and_repair(),
            Err(TaskOperationSnapshotStoreError::InvalidPayload(
                "task_operation_snapshot_journal_corrupt_tail:2".to_owned()
            ))
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn performance_file_task_operation_snapshot_store_roundtrip_stays_within_ci_budget() {
        let path = temp_task_operation_snapshot_path("perf");
        let _ = fs::remove_file(&path);
        let mut engine = TaskOperationEngine::new();
        for index in 0..256 {
            engine
                .submit(
                    &format!("task-store-perf-{index}"),
                    "kamn:did:agent:requester-1",
                    "bounded snapshot benchmark",
                )
                .expect("submit should pass");
        }
        let snapshot = engine.export_snapshot();
        let mut store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        let started = std::time::Instant::now();
        store
            .write(snapshot)
            .expect("write should stay within perf budget");
        let _ = store.read_latest().expect("read should pass");
        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "task operation snapshot store roundtrip exceeded CI budget: {elapsed_millis}ms"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    #[ignore = "scheduled task operation snapshot deep lane"]
    fn performance_task_operation_snapshot_store_deep_lane_stress() {
        let path = temp_task_operation_snapshot_path("deep");
        let _ = fs::remove_file(&path);
        let mut engine = TaskOperationEngine::new();
        for index in 0..6000 {
            engine
                .submit(
                    &format!("task-store-deep-{index}"),
                    "kamn:did:agent:requester-1",
                    "scheduled deep lane benchmark",
                )
                .expect("submit should pass");
        }
        let snapshot = engine.export_snapshot();
        let mut store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        store.write(snapshot).expect("write should pass");
        let _ = store.read_latest().expect("read should pass");
        let _ = fs::remove_file(path);
    }

    fn temp_task_operation_snapshot_path(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-task-operation-snapshot-{tag}-{nonce}.log"))
    }

    fn temp_task_operation_snapshot_journal_path(path: &std::path::Path) -> PathBuf {
        let mut journal = path.as_os_str().to_os_string();
        journal.push(".journal");
        PathBuf::from(journal)
    }
}
