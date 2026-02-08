use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    Submitted,
    Accepted,
    Delegated,
    InProgress,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskTransition {
    Accept,
    Delegate,
    StartWork,
    Block,
    Complete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLifecycle {
    task_id: String,
    state: TaskState,
    history: Vec<TaskState>,
}

impl TaskLifecycle {
    pub fn new(task_id: &str) -> Result<Self, TaskLifecycleError> {
        if task_id.trim().is_empty() {
            return Err(TaskLifecycleError::EmptyTaskId);
        }
        Ok(Self {
            task_id: task_id.to_owned(),
            state: TaskState::Submitted,
            history: vec![TaskState::Submitted],
        })
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn history(&self) -> Vec<TaskState> {
        self.history.clone()
    }

    pub fn transition(&mut self, transition: TaskTransition) -> Result<(), TaskLifecycleError> {
        if is_terminal(self.state) {
            return Err(TaskLifecycleError::TerminalState(self.state));
        }

        let next = match (self.state, transition) {
            (TaskState::Submitted, TaskTransition::Accept) => TaskState::Accepted,
            (TaskState::Submitted, TaskTransition::Cancel) => TaskState::Cancelled,

            (TaskState::Accepted, TaskTransition::Delegate) => TaskState::Delegated,
            (TaskState::Accepted, TaskTransition::StartWork) => TaskState::InProgress,
            (TaskState::Accepted, TaskTransition::Cancel) => TaskState::Cancelled,

            (TaskState::Delegated, TaskTransition::StartWork) => TaskState::InProgress,
            (TaskState::Delegated, TaskTransition::Cancel) => TaskState::Cancelled,

            (TaskState::InProgress, TaskTransition::Block) => TaskState::Blocked,
            (TaskState::InProgress, TaskTransition::Complete) => TaskState::Completed,
            (TaskState::InProgress, TaskTransition::Fail) => TaskState::Failed,
            (TaskState::InProgress, TaskTransition::Cancel) => TaskState::Cancelled,

            (TaskState::Blocked, TaskTransition::StartWork) => TaskState::InProgress,
            (TaskState::Blocked, TaskTransition::Fail) => TaskState::Failed,
            (TaskState::Blocked, TaskTransition::Cancel) => TaskState::Cancelled,

            _ => {
                return Err(TaskLifecycleError::InvalidTransition {
                    from: self.state,
                    transition,
                });
            }
        };

        self.state = next;
        self.history.push(next);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskLifecycleError {
    EmptyTaskId,
    InvalidTransition {
        from: TaskState,
        transition: TaskTransition,
    },
    TerminalState(TaskState),
}

impl fmt::Display for TaskLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTaskId => write!(f, "task_id must not be empty"),
            Self::InvalidTransition { from, transition } => {
                write!(
                    f,
                    "invalid task transition from {:?} via {:?}",
                    from, transition
                )
            }
            Self::TerminalState(state) => write!(f, "task is in terminal state: {state:?}"),
        }
    }
}

impl std::error::Error for TaskLifecycleError {}

fn is_terminal(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Failed | TaskState::Cancelled
    )
}

#[cfg(test)]
mod tests {
    use super::{TaskLifecycle, TaskLifecycleError, TaskState, TaskTransition};

    #[test]
    fn new_rejects_empty_task_id() {
        assert_eq!(TaskLifecycle::new(""), Err(TaskLifecycleError::EmptyTaskId));
    }

    #[test]
    fn blocked_can_return_to_in_progress() {
        let mut lifecycle = match TaskLifecycle::new("task-1") {
            Ok(value) => value,
            Err(error) => panic!("init failed: {error}"),
        };
        assert!(lifecycle.transition(TaskTransition::Accept).is_ok());
        assert!(lifecycle.transition(TaskTransition::StartWork).is_ok());
        assert!(lifecycle.transition(TaskTransition::Block).is_ok());
        assert!(lifecycle.transition(TaskTransition::StartWork).is_ok());
        assert_eq!(lifecycle.state(), TaskState::InProgress);
    }
}
