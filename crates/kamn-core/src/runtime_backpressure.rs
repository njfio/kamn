use super::{is_valid_kamn_did, PeerLifecycleState};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Runtime backpressure action.
pub enum RuntimeBackpressureAction {
    /// Accept.
    Accept,
    /// Slow producer.
    SlowProducer,
    /// Reject new enqueue.
    RejectNewEnqueue,
    /// Purge stale peer queue.
    PurgeStalePeerQueue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime backpressure decision.
pub struct RuntimeBackpressureDecision {
    /// Action.
    pub action: RuntimeBackpressureAction,
    /// Queue utilization per mille.
    pub queue_utilization_per_mille: u16,
    /// Stale peer queue.
    pub stale_peer_queue: bool,
}

impl RuntimeBackpressureDecision {
    /// Returns deterministic action reason code for telemetry and audits.
    pub fn reason_code(&self) -> &'static str {
        match self.action {
            RuntimeBackpressureAction::Accept => "runtime_backpressure_accept",
            RuntimeBackpressureAction::SlowProducer => "runtime_backpressure_slow_producer",
            RuntimeBackpressureAction::RejectNewEnqueue => {
                "runtime_backpressure_reject_new_enqueue"
            }
            RuntimeBackpressureAction::PurgeStalePeerQueue => {
                "runtime_backpressure_purge_stale_peer_queue"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime backpressure policy.
pub struct RuntimeBackpressurePolicy {
    slow_threshold_per_mille: u16,
    reject_threshold_per_mille: u16,
    purge_disconnected_with_pending_queue: bool,
}

impl RuntimeBackpressurePolicy {
    /// Handles new.
    pub fn new(
        slow_threshold_per_mille: u16,
        reject_threshold_per_mille: u16,
        purge_disconnected_with_pending_queue: bool,
    ) -> Result<Self, RuntimeBackpressureError> {
        if slow_threshold_per_mille == 0 || slow_threshold_per_mille > 1000 {
            return Err(RuntimeBackpressureError::InvalidThresholdRange {
                field: "slow_threshold_per_mille",
                found: slow_threshold_per_mille,
            });
        }
        if reject_threshold_per_mille == 0 || reject_threshold_per_mille > 1000 {
            return Err(RuntimeBackpressureError::InvalidThresholdRange {
                field: "reject_threshold_per_mille",
                found: reject_threshold_per_mille,
            });
        }
        if slow_threshold_per_mille >= reject_threshold_per_mille {
            return Err(RuntimeBackpressureError::InvalidThresholdOrder {
                slow_threshold_per_mille,
                reject_threshold_per_mille,
            });
        }

        Ok(Self {
            slow_threshold_per_mille,
            reject_threshold_per_mille,
            purge_disconnected_with_pending_queue,
        })
    }

    /// Handles slow threshold per mille.
    pub fn slow_threshold_per_mille(&self) -> u16 {
        self.slow_threshold_per_mille
    }

    /// Handles reject threshold per mille.
    pub fn reject_threshold_per_mille(&self) -> u16 {
        self.reject_threshold_per_mille
    }

    /// Handles purge disconnected with pending queue.
    pub fn purge_disconnected_with_pending_queue(&self) -> bool {
        self.purge_disconnected_with_pending_queue
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime backpressure input.
pub struct RuntimeBackpressureInput {
    peer_id: String,
    queue_depth: usize,
    queue_capacity: usize,
    lifecycle_state: PeerLifecycleState,
}

impl RuntimeBackpressureInput {
    /// Handles new.
    pub fn new(
        peer_id: &str,
        queue_depth: usize,
        queue_capacity: usize,
        lifecycle_state: PeerLifecycleState,
    ) -> Result<Self, RuntimeBackpressureError> {
        if !is_valid_kamn_did(peer_id) {
            return Err(RuntimeBackpressureError::InvalidPeerId);
        }
        if queue_capacity == 0 {
            return Err(RuntimeBackpressureError::InvalidQueueCapacity {
                capacity: queue_capacity,
            });
        }
        if queue_depth > queue_capacity {
            return Err(RuntimeBackpressureError::QueueDepthExceedsCapacity {
                depth: queue_depth,
                capacity: queue_capacity,
            });
        }

        Ok(Self {
            peer_id: peer_id.to_owned(),
            queue_depth,
            queue_capacity,
            lifecycle_state,
        })
    }

    /// Handles peer id.
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Handles queue depth.
    pub fn queue_depth(&self) -> usize {
        self.queue_depth
    }

    /// Handles queue capacity.
    pub fn queue_capacity(&self) -> usize {
        self.queue_capacity
    }

    /// Handles lifecycle state.
    pub fn lifecycle_state(&self) -> PeerLifecycleState {
        self.lifecycle_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime backpressure error.
pub enum RuntimeBackpressureError {
    /// Invalid threshold range.
    InvalidThresholdRange {
        /// Field.
        field: &'static str,
        /// Found.
        found: u16,
    },
    /// Invalid threshold order.
    InvalidThresholdOrder {
        /// Slow threshold per mille.
        slow_threshold_per_mille: u16,
        /// Reject threshold per mille.
        reject_threshold_per_mille: u16,
    },
    /// Invalid peer id.
    InvalidPeerId,
    /// Invalid queue capacity.
    InvalidQueueCapacity {
        /// Capacity.
        capacity: usize,
    },
    /// Queue depth exceeds capacity.
    QueueDepthExceedsCapacity {
        /// Depth.
        depth: usize,
        /// Capacity.
        capacity: usize,
    },
}

impl RuntimeBackpressureError {
    /// Returns deterministic reason code for validation and policy output.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidThresholdRange { .. } => "runtime_backpressure_threshold_range_invalid",
            Self::InvalidThresholdOrder { .. } => "runtime_backpressure_threshold_order_invalid",
            Self::InvalidPeerId => "runtime_backpressure_peer_id_invalid",
            Self::InvalidQueueCapacity { .. } => "runtime_backpressure_queue_capacity_invalid",
            Self::QueueDepthExceedsCapacity { .. } => {
                "runtime_backpressure_queue_depth_exceeds_capacity"
            }
        }
    }
}

impl Display for RuntimeBackpressureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidThresholdRange { field, found } => write!(
                f,
                "runtime backpressure threshold {field} must be in 1..=1000 (got {found})"
            ),
            Self::InvalidThresholdOrder {
                slow_threshold_per_mille,
                reject_threshold_per_mille,
            } => write!(
                f,
                "runtime backpressure threshold order is invalid: slow {slow_threshold_per_mille}, reject {reject_threshold_per_mille}"
            ),
            Self::InvalidPeerId => write!(f, "runtime backpressure peer id must be a valid DID"),
            Self::InvalidQueueCapacity { capacity } => write!(
                f,
                "runtime backpressure queue capacity must be at least 1 (got {capacity})"
            ),
            Self::QueueDepthExceedsCapacity { depth, capacity } => write!(
                f,
                "runtime backpressure queue depth exceeds capacity: depth {depth}, capacity {capacity}"
            ),
        }
    }
}

impl Error for RuntimeBackpressureError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Deterministic backpressure controller.
pub struct DeterministicBackpressureController {
    policy: RuntimeBackpressurePolicy,
}

impl DeterministicBackpressureController {
    /// Handles new.
    pub fn new(policy: RuntimeBackpressurePolicy) -> Self {
        Self { policy }
    }

    /// Handles evaluate.
    pub fn evaluate(
        &self,
        input: RuntimeBackpressureInput,
    ) -> Result<RuntimeBackpressureDecision, RuntimeBackpressureError> {
        let utilization = queue_utilization_per_mille(input.queue_depth, input.queue_capacity);
        let stale_peer_queue = input.lifecycle_state == PeerLifecycleState::Disconnected
            && input.queue_depth > 0
            && self.policy.purge_disconnected_with_pending_queue;

        let action = if stale_peer_queue {
            RuntimeBackpressureAction::PurgeStalePeerQueue
        } else if utilization >= self.policy.reject_threshold_per_mille {
            RuntimeBackpressureAction::RejectNewEnqueue
        } else if utilization >= self.policy.slow_threshold_per_mille {
            RuntimeBackpressureAction::SlowProducer
        } else {
            RuntimeBackpressureAction::Accept
        };

        Ok(RuntimeBackpressureDecision {
            action,
            queue_utilization_per_mille: utilization,
            stale_peer_queue,
        })
    }
}

fn queue_utilization_per_mille(queue_depth: usize, queue_capacity: usize) -> u16 {
    ((queue_depth as u128) * 1000 / queue_capacity as u128) as u16
}

#[cfg(test)]
mod tests {
    use super::{
        DeterministicBackpressureController, RuntimeBackpressureAction, RuntimeBackpressureInput,
        RuntimeBackpressurePolicy,
    };
    use crate::runtime::PeerLifecycleState;

    #[test]
    fn unit_runtime_backpressure_policy_accepts_valid_thresholds() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        assert_eq!(policy.slow_threshold_per_mille(), 700);
        assert_eq!(policy.reject_threshold_per_mille(), 900);
        assert!(policy.purge_disconnected_with_pending_queue());
    }

    #[test]
    fn regression_runtime_backpressure_rejects_invalid_threshold_order() {
        // Regression: #2832
        let error = RuntimeBackpressurePolicy::new(900, 900, true).expect_err("invalid policy");
        assert_eq!(
            error.reason_code(),
            "runtime_backpressure_threshold_order_invalid"
        );
    }

    #[test]
    fn functional_runtime_backpressure_marks_disconnected_pending_queue_as_stale() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let input = RuntimeBackpressureInput::new(
            "kamn:did:peer:stale-a",
            8,
            10,
            PeerLifecycleState::Disconnected,
        )
        .expect("valid input");
        let decision = controller.evaluate(input).expect("backpressure decision");
        assert_eq!(
            decision.action,
            RuntimeBackpressureAction::PurgeStalePeerQueue
        );
        assert_eq!(
            decision.reason_code(),
            "runtime_backpressure_purge_stale_peer_queue"
        );
    }
}
