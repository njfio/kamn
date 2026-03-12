use super::super::*;
use super::lifecycle::PeerLifecycleState;
#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime queue error.
pub enum RuntimeQueueError {
    /// Invalid capacity.
    InvalidCapacity {
        /// Capacity.
        capacity: usize,
    },
    /// Overflow.
    Overflow {
        /// Capacity.
        capacity: usize,
        /// Attempted len.
        attempted_len: usize,
    },
    /// Backpressure input validation failed.
    BackpressureInput(RuntimeBackpressureError),
    /// Backpressure rejected enqueue for saturation.
    BackpressureRejected {
        /// Deterministic reason code for operational telemetry.
        reason_code: &'static str,
        /// Queue utilization at decision time.
        queue_utilization_per_mille: u16,
    },
    /// Backpressure purged stale disconnected queue.
    BackpressurePurgedStalePeerQueue {
        /// Deterministic reason code for operational telemetry.
        reason_code: &'static str,
        /// Number of queued items purged.
        purged_entries: usize,
    },
}

impl RuntimeQueueError {
    /// Returns deterministic queue/backpressure reason codes.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidCapacity { .. } => "runtime_queue_invalid_capacity",
            Self::Overflow { .. } => "runtime_queue_overflow",
            Self::BackpressureInput(error) => error.reason_code(),
            Self::BackpressureRejected { reason_code, .. } => reason_code,
            Self::BackpressurePurgedStalePeerQueue { reason_code, .. } => reason_code,
        }
    }
}

impl Display for RuntimeQueueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&runtime_queue_error_message(self))
    }
}

impl Error for RuntimeQueueError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Bounded runtime queue.
pub struct BoundedRuntimeQueue<T> {
    capacity: usize,
    entries: VecDeque<T>,
}

impl<T> BoundedRuntimeQueue<T> {
    /// Handles new.
    pub fn new(capacity: usize) -> Result<Self, RuntimeQueueError> {
        if capacity == 0 {
            return Err(RuntimeQueueError::InvalidCapacity { capacity });
        }
        Ok(Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        })
    }

    /// Handles capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Handles len.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Handles is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Handles enqueue.
    pub fn enqueue(&mut self, item: T) -> Result<(), RuntimeQueueError> {
        if self.entries.len() >= self.capacity {
            return Err(RuntimeQueueError::Overflow {
                capacity: self.capacity,
                attempted_len: self.entries.len() + 1,
            });
        }
        self.entries.push_back(item);
        Ok(())
    }

    /// Evaluates deterministic backpressure and applies action before queue mutation.
    pub fn enqueue_with_backpressure(
        &mut self,
        item: T,
        controller: &DeterministicBackpressureController,
        peer_id: &str,
        lifecycle_state: PeerLifecycleState,
    ) -> Result<RuntimeBackpressureDecision, RuntimeQueueError> {
        let decision = build_backpressure_decision(
            controller,
            peer_id,
            self.entries.len(),
            self.capacity,
            lifecycle_state,
        )?;
        apply_backpressure_decision(self, item, decision)
    }

    /// Handles dequeue.
    pub fn dequeue(&mut self) -> Option<T> {
        self.entries.pop_front()
    }
}

fn runtime_queue_error_message(error: &RuntimeQueueError) -> String {
    match error {
        RuntimeQueueError::InvalidCapacity { capacity } => {
            format!("runtime queue capacity must be at least 1 (got {capacity})")
        }
        RuntimeQueueError::Overflow {
            capacity,
            attempted_len,
        } => format!("runtime queue overflow: capacity {capacity}, attempted length {attempted_len}"),
        RuntimeQueueError::BackpressureInput(source) => source.to_string(),
        RuntimeQueueError::BackpressureRejected {
            queue_utilization_per_mille,
            ..
        } => format!(
            "runtime queue enqueue rejected by backpressure at {queue_utilization_per_mille} per mille utilization"
        ),
        RuntimeQueueError::BackpressurePurgedStalePeerQueue { purged_entries, .. } => format!(
            "runtime queue stale peer purge triggered by backpressure; purged {purged_entries} queued entries"
        ),
    }
}

fn build_backpressure_decision(
    controller: &DeterministicBackpressureController,
    peer_id: &str,
    current_len: usize,
    capacity: usize,
    lifecycle_state: PeerLifecycleState,
) -> Result<RuntimeBackpressureDecision, RuntimeQueueError> {
    let input = RuntimeBackpressureInput::new(peer_id, current_len, capacity, lifecycle_state)
        .map_err(RuntimeQueueError::BackpressureInput)?;
    controller
        .evaluate(input)
        .map_err(RuntimeQueueError::BackpressureInput)
}

fn apply_backpressure_decision<T>(
    queue: &mut BoundedRuntimeQueue<T>,
    item: T,
    decision: RuntimeBackpressureDecision,
) -> Result<RuntimeBackpressureDecision, RuntimeQueueError> {
    match decision.action {
        RuntimeBackpressureAction::Accept | RuntimeBackpressureAction::SlowProducer => {
            queue.enqueue(item)?;
            Ok(decision)
        }
        RuntimeBackpressureAction::RejectNewEnqueue => {
            Err(RuntimeQueueError::BackpressureRejected {
                reason_code: decision.reason_code(),
                queue_utilization_per_mille: decision.queue_utilization_per_mille,
            })
        }
        RuntimeBackpressureAction::PurgeStalePeerQueue => purge_stale_queue(queue, decision),
    }
}

fn purge_stale_queue<T>(
    queue: &mut BoundedRuntimeQueue<T>,
    decision: RuntimeBackpressureDecision,
) -> Result<RuntimeBackpressureDecision, RuntimeQueueError> {
    let purged_entries = queue.entries.len();
    queue.entries.clear();
    Err(RuntimeQueueError::BackpressurePurgedStalePeerQueue {
        reason_code: decision.reason_code(),
        purged_entries,
    })
}
