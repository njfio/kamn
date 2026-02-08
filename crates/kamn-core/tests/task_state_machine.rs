use kamn_core::{TaskLifecycle, TaskLifecycleError, TaskState, TaskTransition};

#[test]
fn new_task_starts_submitted_with_history() {
    let lifecycle = TaskLifecycle::new("task-1").expect("task lifecycle should initialize");
    assert_eq!(lifecycle.state(), TaskState::Submitted);
    assert_eq!(lifecycle.history(), vec![TaskState::Submitted]);
}

#[test]
fn legal_transition_flow_reaches_completed() {
    let mut lifecycle = TaskLifecycle::new("task-2").expect("task lifecycle should initialize");

    lifecycle
        .transition(TaskTransition::Accept)
        .expect("submit->accept should succeed");
    lifecycle
        .transition(TaskTransition::Delegate)
        .expect("accept->delegate should succeed");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("delegate->in_progress should succeed");
    lifecycle
        .transition(TaskTransition::Block)
        .expect("in_progress->blocked should succeed");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("blocked->in_progress should succeed");
    lifecycle
        .transition(TaskTransition::Complete)
        .expect("in_progress->completed should succeed");

    assert_eq!(lifecycle.state(), TaskState::Completed);
    assert_eq!(
        lifecycle.history(),
        vec![
            TaskState::Submitted,
            TaskState::Accepted,
            TaskState::Delegated,
            TaskState::InProgress,
            TaskState::Blocked,
            TaskState::InProgress,
            TaskState::Completed,
        ]
    );
}

#[test]
fn invalid_transition_is_rejected() {
    let mut lifecycle = TaskLifecycle::new("task-3").expect("task lifecycle should initialize");
    assert_eq!(
        lifecycle.transition(TaskTransition::Complete),
        Err(TaskLifecycleError::InvalidTransition {
            from: TaskState::Submitted,
            transition: TaskTransition::Complete,
        })
    );
}

#[test]
fn terminal_states_reject_follow_up_transitions() {
    let mut completed = TaskLifecycle::new("task-4").expect("task lifecycle should initialize");
    completed
        .transition(TaskTransition::Accept)
        .expect("accept should succeed");
    completed
        .transition(TaskTransition::StartWork)
        .expect("start should succeed");
    completed
        .transition(TaskTransition::Complete)
        .expect("complete should succeed");
    assert_eq!(
        completed.transition(TaskTransition::Fail),
        Err(TaskLifecycleError::TerminalState(TaskState::Completed))
    );
}

#[test]
fn cancelled_task_cannot_be_completed() {
    let mut lifecycle = TaskLifecycle::new("task-5").expect("task lifecycle should initialize");
    lifecycle
        .transition(TaskTransition::Cancel)
        .expect("submit->cancel should succeed");

    // Regression: #127
    assert_eq!(
        lifecycle.transition(TaskTransition::Complete),
        Err(TaskLifecycleError::TerminalState(TaskState::Cancelled))
    );
}

#[test]
fn input_required_can_resume_to_in_progress_before_completion() {
    let mut lifecycle = TaskLifecycle::new("task-6").expect("task lifecycle should initialize");
    lifecycle
        .transition(TaskTransition::Accept)
        .expect("accept should succeed");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("start should succeed");
    lifecycle
        .transition(TaskTransition::RequestInput)
        .expect("in_progress->input_required should succeed");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("input_required->in_progress should succeed");
    lifecycle
        .transition(TaskTransition::Complete)
        .expect("in_progress->complete should succeed");

    assert_eq!(lifecycle.state(), TaskState::Completed);
}

#[test]
fn regression_input_required_cannot_complete_without_resume() {
    // Regression: #573
    let mut lifecycle = TaskLifecycle::new("task-7").expect("task lifecycle should initialize");
    lifecycle
        .transition(TaskTransition::Accept)
        .expect("accept should succeed");
    lifecycle
        .transition(TaskTransition::StartWork)
        .expect("start should succeed");
    lifecycle
        .transition(TaskTransition::RequestInput)
        .expect("request input should succeed");

    assert_eq!(
        lifecycle.transition(TaskTransition::Complete),
        Err(TaskLifecycleError::InvalidTransition {
            from: TaskState::InputRequired,
            transition: TaskTransition::Complete,
        })
    );
}
