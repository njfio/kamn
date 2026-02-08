use crate::config::{NodeConfig, NodeRole};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerLifecycleState {
    Disconnected,
    Connecting,
    Active,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerLifecycleEvent {
    StartConnect,
    HandshakeSucceeded,
    HeartbeatMissed,
    HeartbeatRestored,
    Disconnect,
    Rejoin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeLifecycleError {
    InvalidPeerId,
    InvalidTransition {
        from: PeerLifecycleState,
        event: PeerLifecycleEvent,
    },
}

impl Display for RuntimeLifecycleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeerId => write!(f, "runtime peer id cannot be empty"),
            Self::InvalidTransition { from, event } => {
                write!(
                    f,
                    "invalid peer lifecycle transition from {from:?} via {event:?}"
                )
            }
        }
    }
}

impl Error for RuntimeLifecycleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycle {
    peer_id: String,
    state: PeerLifecycleState,
}

impl PeerLifecycle {
    pub fn new(peer_id: &str) -> Result<Self, RuntimeLifecycleError> {
        if peer_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::InvalidPeerId);
        }
        Ok(Self {
            peer_id: peer_id.to_owned(),
            state: PeerLifecycleState::Disconnected,
        })
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn state(&self) -> PeerLifecycleState {
        self.state
    }

    pub fn transition(
        &mut self,
        event: PeerLifecycleEvent,
    ) -> Result<PeerLifecycleState, RuntimeLifecycleError> {
        let Some(next_state) = next_peer_state(self.state, event) else {
            return Err(RuntimeLifecycleError::InvalidTransition {
                from: self.state,
                event,
            });
        };
        self.state = next_state;
        Ok(next_state)
    }
}

fn next_peer_state(
    from: PeerLifecycleState,
    event: PeerLifecycleEvent,
) -> Option<PeerLifecycleState> {
    match (from, event) {
        (PeerLifecycleState::Disconnected, PeerLifecycleEvent::StartConnect)
        | (PeerLifecycleState::Disconnected, PeerLifecycleEvent::Rejoin) => {
            Some(PeerLifecycleState::Connecting)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::HandshakeSucceeded) => {
            Some(PeerLifecycleState::Active)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Active, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Degraded, PeerLifecycleEvent::Disconnect) => {
            Some(PeerLifecycleState::Disconnected)
        }
        (PeerLifecycleState::Active, PeerLifecycleEvent::HeartbeatMissed) => {
            Some(PeerLifecycleState::Degraded)
        }
        (PeerLifecycleState::Degraded, PeerLifecycleEvent::HeartbeatRestored) => {
            Some(PeerLifecycleState::Active)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeQueueError {
    InvalidCapacity {
        capacity: usize,
    },
    Overflow {
        capacity: usize,
        attempted_len: usize,
    },
}

impl Display for RuntimeQueueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCapacity { capacity } => {
                write!(
                    f,
                    "runtime queue capacity must be at least 1 (got {capacity})"
                )
            }
            Self::Overflow {
                capacity,
                attempted_len,
            } => write!(
                f,
                "runtime queue overflow: capacity {capacity}, attempted length {attempted_len}"
            ),
        }
    }
}

impl Error for RuntimeQueueError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedRuntimeQueue<T> {
    capacity: usize,
    entries: VecDeque<T>,
}

impl<T> BoundedRuntimeQueue<T> {
    pub fn new(capacity: usize) -> Result<Self, RuntimeQueueError> {
        if capacity == 0 {
            return Err(RuntimeQueueError::InvalidCapacity { capacity });
        }
        Ok(Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

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

    pub fn dequeue(&mut self) -> Option<T> {
        self.entries.pop_front()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWiring {
    pub common_components: Vec<&'static str>,
    pub role_components: Vec<&'static str>,
}

impl RuntimeWiring {
    pub fn all_components(&self) -> Vec<&'static str> {
        let mut components = self.common_components.clone();
        components.extend(self.role_components.iter().copied());
        components
    }
}

pub fn build_runtime_wiring(config: &NodeConfig) -> RuntimeWiring {
    let common_components = vec!["state-store", "message-router", "audit-log", "api-surface"];

    let role_components = match config.role {
        NodeRole::Processor => vec!["mempool", "executor", "block-producer"],
        NodeRole::Listener => vec!["external-listener", "event-normalizer"],
        NodeRole::Approver => vec!["quorum-approver", "outbound-authorizer"],
    };

    RuntimeWiring {
        common_components,
        role_components,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_runtime_wiring, BoundedRuntimeQueue, PeerLifecycle, PeerLifecycleEvent,
        PeerLifecycleState, RuntimeLifecycleError, RuntimeQueueError,
    };
    use crate::config::{NodeConfig, NodeRole, SyncMode};

    fn sample_config(role: NodeRole) -> NodeConfig {
        NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        }
    }

    #[test]
    fn processor_wiring_contains_block_producer() {
        let wiring = build_runtime_wiring(&sample_config(NodeRole::Processor));
        assert!(wiring.role_components.contains(&"block-producer"));
    }

    #[test]
    fn listener_wiring_contains_external_listener() {
        let wiring = build_runtime_wiring(&sample_config(NodeRole::Listener));
        assert!(wiring.role_components.contains(&"external-listener"));
    }

    #[test]
    fn approver_wiring_contains_quorum_approver() {
        let wiring = build_runtime_wiring(&sample_config(NodeRole::Approver));
        assert!(wiring.role_components.contains(&"quorum-approver"));
    }

    #[test]
    fn functional_peer_lifecycle_allows_connect_heartbeat_recover_disconnect_flow() {
        let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
        assert_eq!(lifecycle.peer_id(), "peer-1");
        assert_eq!(lifecycle.state(), PeerLifecycleState::Disconnected);
        assert!(lifecycle
            .transition(PeerLifecycleEvent::StartConnect)
            .is_ok());
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HeartbeatMissed)
            .is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Degraded);
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HeartbeatRestored)
            .is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Active);
        assert!(lifecycle.transition(PeerLifecycleEvent::Disconnect).is_ok());
        assert_eq!(lifecycle.state(), PeerLifecycleState::Disconnected);
    }

    #[test]
    fn integration_bounded_runtime_queue_preserves_fifo_until_capacity() {
        let mut queue = BoundedRuntimeQueue::new(2).expect("queue should build");
        assert_eq!(queue.capacity(), 2);
        assert!(queue.is_empty());
        assert!(queue.enqueue("evt-1".to_owned()).is_ok());
        assert!(queue.enqueue("evt-2".to_owned()).is_ok());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.dequeue(), Some("evt-1".to_owned()));
        assert_eq!(queue.dequeue(), Some("evt-2".to_owned()));
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn unit_rejects_invalid_peer_lifecycle_transition() {
        let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
        let error = lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .expect_err("handshake cannot complete before connect");
        assert_eq!(
            error,
            RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Disconnected,
                event: PeerLifecycleEvent::HandshakeSucceeded
            }
        );
    }

    #[test]
    fn regression_rejoin_without_disconnect_is_rejected() {
        // Regression: #324
        let mut lifecycle = PeerLifecycle::new("peer-1").expect("valid peer id");
        assert!(lifecycle
            .transition(PeerLifecycleEvent::StartConnect)
            .is_ok());
        assert!(lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .is_ok());
        let error = lifecycle
            .transition(PeerLifecycleEvent::Rejoin)
            .expect_err("rejoin should require disconnected state");
        assert_eq!(
            error,
            RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Active,
                event: PeerLifecycleEvent::Rejoin
            }
        );
    }

    #[test]
    fn regression_queue_overflow_rejects_new_event() {
        // Regression: #324
        let mut queue = BoundedRuntimeQueue::new(1).expect("queue should build");
        assert!(queue.enqueue("evt-1".to_owned()).is_ok());
        let error = queue
            .enqueue("evt-2".to_owned())
            .expect_err("second enqueue must overflow");
        assert_eq!(
            error,
            RuntimeQueueError::Overflow {
                capacity: 1,
                attempted_len: 2
            }
        );
    }

    #[test]
    fn unit_rejects_empty_peer_id() {
        assert_eq!(
            PeerLifecycle::new(""),
            Err(RuntimeLifecycleError::InvalidPeerId)
        );
    }

    #[test]
    fn unit_rejects_zero_queue_capacity() {
        assert_eq!(
            BoundedRuntimeQueue::<String>::new(0),
            Err(RuntimeQueueError::InvalidCapacity { capacity: 0 })
        );
    }
}
