use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Expected outcome category for a lifecycle regression replay case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerLifecycleRegressionExpectedOutcome {
    /// Replay should complete and end on the provided lifecycle state.
    FinalState(PeerLifecycleState),
    /// Replay should fail closed with the provided transition error.
    TransitionError(RuntimeLifecycleError),
}

/// Deterministic lifecycle regression replay case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycleRegressionCase {
    case_id: String,
    events: Vec<PeerLifecycleEvent>,
    expected_outcome: PeerLifecycleRegressionExpectedOutcome,
}

impl PeerLifecycleRegressionCase {
    /// Builds a validated lifecycle regression replay case.
    pub fn new(
        case_id: &str,
        events: Vec<PeerLifecycleEvent>,
        expected_outcome: PeerLifecycleRegressionExpectedOutcome,
    ) -> Result<Self, PeerLifecycleRegressionError> {
        if case_id.trim().is_empty() {
            return Err(PeerLifecycleRegressionError::EmptyCaseId);
        }
        if events.is_empty() {
            return Err(PeerLifecycleRegressionError::EmptyEventSequence);
        }
        Ok(Self {
            case_id: case_id.to_owned(),
            events,
            expected_outcome,
        })
    }

    /// Returns deterministic replay case id.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns replay event sequence.
    pub fn events(&self) -> &[PeerLifecycleEvent] {
        &self.events
    }

    /// Returns expected replay outcome.
    pub fn expected_outcome(&self) -> &PeerLifecycleRegressionExpectedOutcome {
        &self.expected_outcome
    }
}

/// Deterministic lifecycle regression replay outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycleRegressionOutcome {
    case_id: String,
    final_state: Option<PeerLifecycleState>,
    transition_error_reason_code: Option<&'static str>,
}

impl PeerLifecycleRegressionOutcome {
    /// Returns replay case id.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns final lifecycle state when replay succeeded.
    pub fn final_state(&self) -> Option<PeerLifecycleState> {
        self.final_state
    }

    /// Returns deterministic transition error reason code when replay failed.
    pub fn transition_error_reason_code(&self) -> Option<&'static str> {
        self.transition_error_reason_code
    }
}

/// Lifecycle regression replay error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerLifecycleRegressionError {
    /// Case id is empty.
    EmptyCaseId,
    /// Event sequence is empty.
    EmptyEventSequence,
    /// Lifecycle construction or transition returned a runtime error.
    Lifecycle(RuntimeLifecycleError),
    /// Final lifecycle state differs from expected deterministic state.
    ExpectedFinalStateMismatch {
        /// Case id.
        case_id: String,
        /// Expected state.
        expected: PeerLifecycleState,
        /// Observed state.
        found: PeerLifecycleState,
    },
    /// Transition error occurred when case expected a final-state result.
    UnexpectedTransitionError {
        /// Case id.
        case_id: String,
        /// Observed transition error.
        found: RuntimeLifecycleError,
    },
    /// Expected transition-error contract differs from observed result.
    ExpectedTransitionErrorMismatch {
        /// Case id.
        case_id: String,
        /// Expected transition error.
        expected: RuntimeLifecycleError,
        /// Observed transition error, if one occurred.
        found: Option<RuntimeLifecycleError>,
    },
}

impl Display for PeerLifecycleRegressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCaseId => write!(f, "lifecycle regression case id cannot be empty"),
            Self::EmptyEventSequence => {
                write!(f, "lifecycle regression event sequence cannot be empty")
            }
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::ExpectedFinalStateMismatch {
                case_id,
                expected,
                found,
            } => write!(
                f,
                "lifecycle regression case {case_id} expected final state {expected:?}, found {found:?}"
            ),
            Self::UnexpectedTransitionError { case_id, found } => write!(
                f,
                "lifecycle regression case {case_id} observed unexpected transition error {found:?}"
            ),
            Self::ExpectedTransitionErrorMismatch {
                case_id,
                expected,
                found,
            } => write!(
                f,
                "lifecycle regression case {case_id} expected transition error {expected:?}, found {found:?}"
            ),
        }
    }
}

impl Error for PeerLifecycleRegressionError {}

/// Builds deterministic default lifecycle regression corpus for libp2p transport transitions.
pub fn build_libp2p_lifecycle_regression_corpus() -> Vec<PeerLifecycleRegressionCase> {
    vec![
        PeerLifecycleRegressionCase {
            case_id: "connect_handshake_disconnect".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Disconnected,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "connect_heartbeat_timeout_recovery".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::HeartbeatMissed,
                PeerLifecycleEvent::HeartbeatRestored,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Active,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "connect_drop_rejoin".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
                PeerLifecycleEvent::Rejoin,
                PeerLifecycleEvent::HandshakeSucceeded,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Active,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "invalid_heartbeat_from_disconnected".to_owned(),
            events: vec![PeerLifecycleEvent::HeartbeatMissed],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::TransitionError(
                RuntimeLifecycleError::InvalidTransition {
                    from: PeerLifecycleState::Disconnected,
                    event: PeerLifecycleEvent::HeartbeatMissed,
                },
            ),
        },
    ]
}

/// Replays one deterministic lifecycle regression case.
pub fn run_libp2p_lifecycle_regression_case(
    peer_id: &str,
    case: &PeerLifecycleRegressionCase,
) -> Result<PeerLifecycleRegressionOutcome, PeerLifecycleRegressionError> {
    let mut lifecycle =
        PeerLifecycle::new(peer_id).map_err(PeerLifecycleRegressionError::Lifecycle)?;

    let mut observed_error = None;
    let mut observed_state = lifecycle.state();
    for event in case.events() {
        match lifecycle.transition(*event) {
            Ok(next_state) => observed_state = next_state,
            Err(error) => {
                observed_error = Some(error);
                break;
            }
        }
    }

    match case.expected_outcome() {
        PeerLifecycleRegressionExpectedOutcome::FinalState(expected) => {
            if let Some(error) = observed_error {
                return Err(PeerLifecycleRegressionError::UnexpectedTransitionError {
                    case_id: case.case_id().to_owned(),
                    found: error,
                });
            }
            if &observed_state != expected {
                return Err(PeerLifecycleRegressionError::ExpectedFinalStateMismatch {
                    case_id: case.case_id().to_owned(),
                    expected: *expected,
                    found: observed_state,
                });
            }
            Ok(PeerLifecycleRegressionOutcome {
                case_id: case.case_id().to_owned(),
                final_state: Some(observed_state),
                transition_error_reason_code: None,
            })
        }
        PeerLifecycleRegressionExpectedOutcome::TransitionError(expected_error) => {
            let Some(found_error) = observed_error else {
                return Err(
                    PeerLifecycleRegressionError::ExpectedTransitionErrorMismatch {
                        case_id: case.case_id().to_owned(),
                        expected: expected_error.clone(),
                        found: None,
                    },
                );
            };
            if &found_error != expected_error {
                return Err(
                    PeerLifecycleRegressionError::ExpectedTransitionErrorMismatch {
                        case_id: case.case_id().to_owned(),
                        expected: expected_error.clone(),
                        found: Some(found_error),
                    },
                );
            }
            Ok(PeerLifecycleRegressionOutcome {
                case_id: case.case_id().to_owned(),
                final_state: None,
                transition_error_reason_code: Some(found_error.reason_code()),
            })
        }
    }
}

/// Replays deterministic lifecycle regression corpus in the provided order.
pub fn run_libp2p_lifecycle_regression_corpus(
    peer_id: &str,
    corpus: &[PeerLifecycleRegressionCase],
) -> Result<Vec<PeerLifecycleRegressionOutcome>, PeerLifecycleRegressionError> {
    let mut outcomes = Vec::with_capacity(corpus.len());
    for case in corpus {
        outcomes.push(run_libp2p_lifecycle_regression_case(peer_id, case)?);
    }
    Ok(outcomes)
}
