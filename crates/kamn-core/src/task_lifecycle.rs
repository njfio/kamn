//! Task lifecycle state machine and transition evidence contracts.

use std::fmt;

/// Snapshot of task lifecycle state within the task state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    /// Task has been created but not yet accepted.
    Submitted,
    /// Task has been accepted by an assignee.
    Accepted,
    /// Task has been delegated to another assignee.
    Delegated,
    /// Task is actively being worked.
    InProgress,
    /// Task is waiting for additional external input.
    InputRequired,
    /// Task is temporarily blocked.
    Blocked,
    /// Task completed successfully.
    Completed,
    /// Task ended in failure.
    Failed,
    /// Task has been cancelled.
    Cancelled,
}

/// Transition action applied to [`TaskLifecycle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskTransition {
    /// Accept a submitted task.
    Accept,
    /// Delegate an accepted task.
    Delegate,
    /// Start or resume work.
    StartWork,
    /// Request additional input while in progress.
    RequestInput,
    /// Mark an in-progress task as blocked.
    Block,
    /// Complete work successfully.
    Complete,
    /// Mark work as failed.
    Fail,
    /// Cancel the task.
    Cancel,
}

/// Evidence record describing an allowed task transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskTransitionEvidence {
    /// Source state before transition application.
    pub from: TaskState,
    /// Applied transition.
    pub transition: TaskTransition,
    /// Destination state after transition application.
    pub to: TaskState,
    /// Stable reason-code emitted for policy contracts.
    pub reason_code: &'static str,
}

/// Mutable task lifecycle state machine with replayable history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLifecycle {
    task_id: String,
    state: TaskState,
    history: Vec<TaskState>,
}

impl TaskLifecycle {
    /// Create a new lifecycle initialized in [`TaskState::Submitted`].
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

    /// Return the stable task identifier.
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Return the current lifecycle state.
    pub fn state(&self) -> TaskState {
        self.state
    }

    /// Return the replay history as an owned state vector.
    pub fn history(&self) -> Vec<TaskState> {
        self.history.clone()
    }

    /// Apply a transition and mutate lifecycle state if the edge is valid.
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

    /// Apply a transition and emit deterministic transition evidence.
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

    /// Restore a lifecycle from historical states after validating each edge.
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

/// Error variants emitted by [`TaskLifecycle`] transition and restore operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskLifecycleError {
    /// Provided task identifier is empty.
    EmptyTaskId,
    /// Lifecycle history is malformed or not replayable.
    InvalidHistory(String),
    /// Requested transition edge is invalid for current state.
    InvalidTransition {
        /// Source state where invalid transition was requested.
        from: TaskState,
        /// Transition that was rejected.
        transition: TaskTransition,
    },
    /// Transition was requested from a terminal state.
    TerminalState(TaskState),
}

impl TaskLifecycleError {
    /// Return deterministic reason-code used by contract lanes.
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
