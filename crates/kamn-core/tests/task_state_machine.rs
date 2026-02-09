use kamn_core::{TaskLifecycle, TaskLifecycleError, TaskState, TaskTransition};

const TASK_TRANSITIONS: [TaskTransition; 8] = [
    TaskTransition::Accept,
    TaskTransition::Delegate,
    TaskTransition::StartWork,
    TaskTransition::RequestInput,
    TaskTransition::Block,
    TaskTransition::Complete,
    TaskTransition::Fail,
    TaskTransition::Cancel,
];

fn is_terminal_task_state(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Failed | TaskState::Cancelled
    )
}

fn is_legal_task_state_step(from: TaskState, to: TaskState) -> bool {
    matches!(
        (from, to),
        (TaskState::Submitted, TaskState::Accepted)
            | (TaskState::Submitted, TaskState::Cancelled)
            | (TaskState::Accepted, TaskState::Delegated)
            | (TaskState::Accepted, TaskState::InProgress)
            | (TaskState::Accepted, TaskState::Cancelled)
            | (TaskState::Delegated, TaskState::InProgress)
            | (TaskState::Delegated, TaskState::Cancelled)
            | (TaskState::InProgress, TaskState::Blocked)
            | (TaskState::InProgress, TaskState::InputRequired)
            | (TaskState::InProgress, TaskState::Completed)
            | (TaskState::InProgress, TaskState::Failed)
            | (TaskState::InProgress, TaskState::Cancelled)
            | (TaskState::InputRequired, TaskState::InProgress)
            | (TaskState::InputRequired, TaskState::Failed)
            | (TaskState::InputRequired, TaskState::Cancelled)
            | (TaskState::Blocked, TaskState::InProgress)
            | (TaskState::Blocked, TaskState::Failed)
            | (TaskState::Blocked, TaskState::Cancelled)
    )
}

fn for_each_task_transition_sequence(max_len: usize, mut f: impl FnMut(&[TaskTransition])) {
    fn recurse(
        target_len: usize,
        current: &mut Vec<TaskTransition>,
        f: &mut impl FnMut(&[TaskTransition]),
    ) {
        if current.len() == target_len {
            f(current.as_slice());
            return;
        }

        for transition in TASK_TRANSITIONS {
            current.push(transition);
            recurse(target_len, current, f);
            current.pop();
        }
    }

    let mut current = Vec::new();
    for len in 1..=max_len {
        recurse(len, &mut current, &mut f);
    }
}

#[test]
fn task_lifecycle_property_generated_sequences_preserve_transition_contracts() {
    // Keep sequence depth bounded for fast CI while still exploring broad transition permutations.
    for_each_task_transition_sequence(4, |sequence| {
        let mut lifecycle =
            TaskLifecycle::new("task-property").expect("task lifecycle should initialize");
        let mut successful_transitions = 0_usize;

        for transition in sequence {
            let before_state = lifecycle.state();
            let before_history = lifecycle.history();
            match lifecycle.transition(*transition) {
                Ok(()) => {
                    successful_transitions += 1;
                    let after_state = lifecycle.state();
                    assert!(
                        is_legal_task_state_step(before_state, after_state),
                        "successful transition must be legal from {before_state:?} via \
                         {transition:?} to {after_state:?}"
                    );
                    assert_eq!(
                        lifecycle.history().len(),
                        before_history.len() + 1,
                        "successful transition must append one state to history"
                    );
                }
                Err(TaskLifecycleError::InvalidTransition {
                    from,
                    transition: rejected,
                }) => {
                    assert_eq!(from, before_state);
                    assert_eq!(rejected, *transition);
                    assert_eq!(lifecycle.state(), before_state);
                    assert_eq!(lifecycle.history(), before_history);
                }
                Err(TaskLifecycleError::TerminalState(state)) => {
                    assert!(is_terminal_task_state(state));
                    assert_eq!(state, before_state);
                    assert_eq!(lifecycle.state(), before_state);
                    assert_eq!(lifecycle.history(), before_history);
                }
                Err(error) => panic!("unexpected task lifecycle error in property lane: {error:?}"),
            }

            let history = lifecycle.history();
            assert_eq!(history.first(), Some(&TaskState::Submitted));
            assert_eq!(history.last().copied(), Some(lifecycle.state()));
        }

        assert_eq!(
            lifecycle.history().len(),
            successful_transitions + 1,
            "history length must equal initial submitted state plus successful transitions"
        );
    });
}

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
