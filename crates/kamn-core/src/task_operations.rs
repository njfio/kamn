use crate::{AgentDid, TaskLifecycle, TaskLifecycleError, TaskState, TaskTransition};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecord {
    pub task_id: String,
    pub requester: String,
    pub assignee: Option<String>,
    pub description: String,
    pub lifecycle: TaskLifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmTaskDraft {
    pub task_id: String,
    pub requester: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

pub const TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationSnapshot {
    pub schema_version: u16,
    pub tasks: Vec<TaskOperationRecordSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskOperationEngine {
    tasks: BTreeMap<String, TaskOperationRecord>,
    notices_by_task: BTreeMap<String, Vec<TaskOperationNoticeKind>>,
    dependencies_by_task: BTreeMap<String, BTreeSet<String>>,
}

impl TaskOperationEngine {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn task(&self, task_id: &str) -> Result<&TaskOperationRecord, TaskOperationError> {
        self.tasks
            .get(task_id)
            .ok_or_else(|| TaskOperationError::NotFound(task_id.to_owned()))
    }

    pub fn notices(&self, task_id: &str) -> Vec<TaskOperationNoticeKind> {
        self.notices_by_task
            .get(task_id)
            .cloned()
            .unwrap_or_default()
    }

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

#[cfg(test)]
mod tests {
    use super::{SwarmTaskDraft, TaskOperationEngine, TaskOperationError};
    use crate::TaskState;

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
}
