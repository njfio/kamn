use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    Submitted,
    Accepted,
    Delegated,
    InProgress,
    InputRequired,
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
    RequestInput,
    Block,
    Complete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskTransitionEvidence {
    pub from: TaskState,
    pub transition: TaskTransition,
    pub to: TaskState,
    pub reason_code: &'static str,
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

        let next =
            next_state(self.state, transition).ok_or(TaskLifecycleError::InvalidTransition {
                from: self.state,
                transition,
            })?;

        self.state = next;
        self.history.push(next);
        Ok(())
    }

    pub fn transition_with_evidence(
        &mut self,
        transition: TaskTransition,
    ) -> Result<TaskTransitionEvidence, TaskLifecycleError> {
        let from = self.state();
        self.transition(transition)?;
        Ok(TaskTransitionEvidence {
            from,
            transition,
            to: self.state(),
            reason_code: "task_transition_allowed",
        })
    }

    pub fn restore(task_id: &str, history: Vec<TaskState>) -> Result<Self, TaskLifecycleError> {
        if history.is_empty() {
            return Err(TaskLifecycleError::InvalidHistory(
                "history must not be empty".to_owned(),
            ));
        }
        if history[0] != TaskState::Submitted {
            return Err(TaskLifecycleError::InvalidHistory(
                "history must begin with Submitted".to_owned(),
            ));
        }

        let mut lifecycle = Self::new(task_id)?;
        for states in history.windows(2) {
            let from = states[0];
            let to = states[1];
            let transition = transition_between(from, to).ok_or_else(|| {
                TaskLifecycleError::InvalidHistory(format!(
                    "invalid state step from {from:?} to {to:?}"
                ))
            })?;
            lifecycle.transition(transition)?;
        }

        if lifecycle.history != history {
            return Err(TaskLifecycleError::InvalidHistory(
                "restored history does not match replayed history".to_owned(),
            ));
        }
        Ok(lifecycle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskLifecycleError {
    EmptyTaskId,
    InvalidHistory(String),
    InvalidTransition {
        from: TaskState,
        transition: TaskTransition,
    },
    TerminalState(TaskState),
}

impl TaskLifecycleError {
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::EmptyTaskId => "task_id_empty",
            Self::InvalidHistory(_) => "task_history_invalid",
            Self::InvalidTransition { .. } => "task_transition_invalid_edge",
            Self::TerminalState(_) => "task_transition_terminal_state",
        }
    }
}

impl fmt::Display for TaskLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTaskId => write!(f, "task_id must not be empty"),
            Self::InvalidHistory(value) => write!(f, "invalid task history: {value}"),
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

fn next_state(from: TaskState, transition: TaskTransition) -> Option<TaskState> {
    match (from, transition) {
        (TaskState::Submitted, TaskTransition::Accept) => Some(TaskState::Accepted),
        (TaskState::Submitted, TaskTransition::Cancel) => Some(TaskState::Cancelled),

        (TaskState::Accepted, TaskTransition::Delegate) => Some(TaskState::Delegated),
        (TaskState::Accepted, TaskTransition::StartWork) => Some(TaskState::InProgress),
        (TaskState::Accepted, TaskTransition::Cancel) => Some(TaskState::Cancelled),

        (TaskState::Delegated, TaskTransition::StartWork) => Some(TaskState::InProgress),
        (TaskState::Delegated, TaskTransition::Cancel) => Some(TaskState::Cancelled),

        (TaskState::InProgress, TaskTransition::Block) => Some(TaskState::Blocked),
        (TaskState::InProgress, TaskTransition::RequestInput) => Some(TaskState::InputRequired),
        (TaskState::InProgress, TaskTransition::Complete) => Some(TaskState::Completed),
        (TaskState::InProgress, TaskTransition::Fail) => Some(TaskState::Failed),
        (TaskState::InProgress, TaskTransition::Cancel) => Some(TaskState::Cancelled),

        (TaskState::InputRequired, TaskTransition::StartWork) => Some(TaskState::InProgress),
        (TaskState::InputRequired, TaskTransition::Fail) => Some(TaskState::Failed),
        (TaskState::InputRequired, TaskTransition::Cancel) => Some(TaskState::Cancelled),

        (TaskState::Blocked, TaskTransition::StartWork) => Some(TaskState::InProgress),
        (TaskState::Blocked, TaskTransition::Fail) => Some(TaskState::Failed),
        (TaskState::Blocked, TaskTransition::Cancel) => Some(TaskState::Cancelled),

        _ => None,
    }
}

fn transition_between(from: TaskState, to: TaskState) -> Option<TaskTransition> {
    match (from, to) {
        (TaskState::Submitted, TaskState::Accepted) => Some(TaskTransition::Accept),
        (TaskState::Submitted, TaskState::Cancelled) => Some(TaskTransition::Cancel),

        (TaskState::Accepted, TaskState::Delegated) => Some(TaskTransition::Delegate),
        (TaskState::Accepted, TaskState::InProgress) => Some(TaskTransition::StartWork),
        (TaskState::Accepted, TaskState::Cancelled) => Some(TaskTransition::Cancel),

        (TaskState::Delegated, TaskState::InProgress) => Some(TaskTransition::StartWork),
        (TaskState::Delegated, TaskState::Cancelled) => Some(TaskTransition::Cancel),

        (TaskState::InProgress, TaskState::Blocked) => Some(TaskTransition::Block),
        (TaskState::InProgress, TaskState::InputRequired) => Some(TaskTransition::RequestInput),
        (TaskState::InProgress, TaskState::Completed) => Some(TaskTransition::Complete),
        (TaskState::InProgress, TaskState::Failed) => Some(TaskTransition::Fail),
        (TaskState::InProgress, TaskState::Cancelled) => Some(TaskTransition::Cancel),

        (TaskState::InputRequired, TaskState::InProgress) => Some(TaskTransition::StartWork),
        (TaskState::InputRequired, TaskState::Failed) => Some(TaskTransition::Fail),
        (TaskState::InputRequired, TaskState::Cancelled) => Some(TaskTransition::Cancel),

        (TaskState::Blocked, TaskState::InProgress) => Some(TaskTransition::StartWork),
        (TaskState::Blocked, TaskState::Failed) => Some(TaskTransition::Fail),
        (TaskState::Blocked, TaskState::Cancelled) => Some(TaskTransition::Cancel),

        _ => None,
    }
}

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

    #[test]
    fn input_required_can_return_to_in_progress() {
        let mut lifecycle = match TaskLifecycle::new("task-2") {
            Ok(value) => value,
            Err(error) => panic!("init failed: {error}"),
        };
        assert!(lifecycle.transition(TaskTransition::Accept).is_ok());
        assert!(lifecycle.transition(TaskTransition::StartWork).is_ok());
        assert!(lifecycle.transition(TaskTransition::RequestInput).is_ok());
        assert!(lifecycle.transition(TaskTransition::StartWork).is_ok());
        assert_eq!(lifecycle.state(), TaskState::InProgress);
    }

    #[test]
    fn restore_rejects_empty_history() {
        assert_eq!(
            TaskLifecycle::restore("task-restore-1", vec![]),
            Err(TaskLifecycleError::InvalidHistory(
                "history must not be empty".to_owned()
            ))
        );
    }

    #[test]
    fn restore_replays_valid_history() {
        let lifecycle = TaskLifecycle::restore(
            "task-restore-2",
            vec![
                TaskState::Submitted,
                TaskState::Accepted,
                TaskState::InProgress,
                TaskState::Completed,
            ],
        )
        .expect("restore should pass");
        assert_eq!(lifecycle.state(), TaskState::Completed);
        assert_eq!(
            lifecycle.history(),
            vec![
                TaskState::Submitted,
                TaskState::Accepted,
                TaskState::InProgress,
                TaskState::Completed,
            ]
        );
    }
}
