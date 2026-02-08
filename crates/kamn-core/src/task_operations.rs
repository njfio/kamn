use crate::{AgentDid, TaskLifecycle, TaskLifecycleError, TaskTransition};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskOperationNoticeKind {
    Submitted,
    Accepted,
    Delegated,
    Started,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskOperationEngine {
    tasks: BTreeMap<String, TaskOperationRecord>,
    notices_by_task: BTreeMap<String, Vec<TaskOperationNoticeKind>>,
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
        self.push_notice(task_id, TaskOperationNoticeKind::Submitted);
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOperationError {
    NotFound(String),
    DuplicateTaskId(String),
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

#[cfg(test)]
mod tests {
    use super::{TaskOperationEngine, TaskOperationError};
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
}
