use runtime_recovery_guard::{is_valid_kamn_did, is_valid_listener_did};

#[path = "runtime_backpressure.rs"]
mod runtime_backpressure;
#[path = "runtime_network_fault.rs"]
mod runtime_network_fault;
#[path = "runtime_peer_coordination.rs"]
mod runtime_peer_coordination;
#[path = "runtime_phase_coordination.rs"]
mod runtime_phase_coordination;
#[path = "runtime_recovery_guard.rs"]
mod runtime_recovery_guard;
#[path = "runtime_snapshot_store.rs"]
mod runtime_snapshot_store;
#[path = "runtime_state_divergence.rs"]
mod runtime_state_divergence;
#[path = "runtime_transport_coordination.rs"]
mod runtime_transport_coordination;

pub use runtime_backpressure::{
    DeterministicBackpressureController, RuntimeBackpressureAction, RuntimeBackpressureDecision,
    RuntimeBackpressureError, RuntimeBackpressureInput, RuntimeBackpressurePolicy,
};
pub use runtime_network_fault::{
    simulate_daemon_network_fault, DeterministicNetworkFaultSimulator, NetworkFaultSimulationError,
    NetworkFaultSimulationInput, NetworkFaultSimulationReport,
};
pub use runtime_peer_coordination::{
    build_runtime_wiring, AuthenticatedPeerFrame, AuthenticatedPeerFrameError, BoundedRuntimeQueue,
    DeterministicProposalPlanner, PeerFrameAuthenticator, PeerLifecycle, PeerLifecycleEvent,
    PeerLifecycleState, ProposalCandidate, ProposalPlan, ProposalPlannerError,
    RuntimeLifecycleError, RuntimeQueueError, RuntimeWiring,
};
pub use runtime_phase_coordination::{
    authorize_daemon_outbound_action, evaluate_daemon_listener_quorum,
    execute_processor_daemon_tick, ApproverAttestation, ApproverQuorumDecision,
    ApproverQuorumError, ApproverQuorumEvaluator, ApproverQuorumInput, ConstructLockError,
    ConstructLockGuard, ConstructLockLease, ListenerAttestation, ListenerQuorumDecision,
    ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput,
};
pub use runtime_recovery_guard::{
    RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
};
pub use runtime_snapshot_store::{
    FileRuntimeSnapshotStore, InMemoryRuntimeSnapshotStore, RuntimeSnapshot, RuntimeSnapshotStore,
    SnapshotRecoveryResult, SnapshotRestoreError, SnapshotRestoreGuard, SnapshotStoreError,
};
pub use runtime_state_divergence::{
    evaluate_daemon_state_divergence, StateDivergenceError, StateDivergenceEvaluator,
    StateDivergenceEvidence, StateDivergenceReport, StateDivergenceSeverity, StateDivergenceStatus,
    StateDivergenceWatchInput,
};
pub use runtime_transport_coordination::{
    evaluate_daemon_watchdog_anomaly, WatchdogAnomalyError, WatchdogAnomalyEvaluator,
    WatchdogAnomalyKind, WatchdogAnomalyReport, WatchdogAnomalySeverity, WatchdogAnomalyWatchInput,
};

#[cfg(test)]
mod tests {
    use super::{
        authorize_daemon_outbound_action, build_runtime_wiring, evaluate_daemon_state_divergence,
        evaluate_daemon_watchdog_anomaly, execute_processor_daemon_tick, ApproverAttestation,
        ApproverQuorumError, ApproverQuorumEvaluator, ApproverQuorumInput, AuthenticatedPeerFrame,
        AuthenticatedPeerFrameError, BoundedRuntimeQueue, ConstructLockError, ConstructLockGuard,
        DeterministicBackpressureController, DeterministicNetworkFaultSimulator,
        DeterministicProposalPlanner, FileRuntimeSnapshotStore, InMemoryRuntimeSnapshotStore,
        ListenerAttestation, ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput,
        NetworkFaultSimulationError, NetworkFaultSimulationInput, PeerFrameAuthenticator,
        PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate,
        ProposalPlannerError, RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus,
        RejoinAttempt, RuntimeBackpressureAction, RuntimeBackpressureDecision,
        RuntimeBackpressureError, RuntimeBackpressureInput, RuntimeBackpressurePolicy,
        RuntimeLifecycleError, RuntimeQueueError, RuntimeSnapshot, RuntimeSnapshotStore,
        SnapshotRestoreError, SnapshotRestoreGuard, SnapshotStoreError, StateDivergenceError,
        StateDivergenceEvaluator, StateDivergenceSeverity, StateDivergenceStatus,
        StateDivergenceWatchInput, WatchdogAnomalyError, WatchdogAnomalyEvaluator,
        WatchdogAnomalyKind, WatchdogAnomalySeverity, WatchdogAnomalyWatchInput,
    };
    use crate::config::{NodeConfig, NodeRole, SyncMode};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
    fn regression_runtime_source_routes_network_fault_domain_via_dedicated_module() {
        // Regression: #3187
        let runtime_source = include_str!("runtime.rs");
        let declaration = [
            "#[path = \"runtime_network_fault.rs\"]",
            "mod runtime_network_fault;",
        ]
        .join("\n");
        assert!(
            runtime_source.contains(&declaration),
            "expected runtime module declaration for network fault extraction"
        );
        assert!(
            runtime_source.contains("pub use runtime_network_fault::{"),
            "expected runtime re-export surface for extracted network fault APIs"
        );
        for symbol in [
            "simulate_daemon_network_fault",
            "DeterministicNetworkFaultSimulator",
            "NetworkFaultSimulationError",
            "NetworkFaultSimulationInput",
            "NetworkFaultSimulationReport",
        ] {
            assert!(
                runtime_source.contains(symbol),
                "expected runtime network fault re-export to include `{symbol}`"
            );
        }
        assert!(
            runtime_source.contains("};"),
            "expected re-export block terminator to remain present"
        );
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

    #[test]
    fn unit_runtime_backpressure_policy_rejects_invalid_threshold_order() {
        assert_eq!(
            RuntimeBackpressurePolicy::new(900, 900, true),
            Err(RuntimeBackpressureError::InvalidThresholdOrder {
                slow_threshold_per_mille: 900,
                reject_threshold_per_mille: 900
            })
        );
    }

    #[test]
    fn functional_runtime_backpressure_classifies_queue_saturation() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let input = RuntimeBackpressureInput::new(
            "kamn:did:agent:peer-a",
            8,
            10,
            PeerLifecycleState::Active,
        )
        .expect("valid input");
        let decision = controller.evaluate(input).expect("evaluation should pass");
        assert_eq!(decision.action, RuntimeBackpressureAction::SlowProducer);
        assert_eq!(decision.queue_utilization_per_mille, 800);
    }

    #[test]
    fn integration_runtime_backpressure_purges_stale_disconnected_peer_queue() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let input = RuntimeBackpressureInput::new(
            "kamn:did:agent:peer-b",
            3,
            10,
            PeerLifecycleState::Disconnected,
        )
        .expect("valid input");
        let decision = controller.evaluate(input).expect("evaluation should pass");
        assert_eq!(
            decision.action,
            RuntimeBackpressureAction::PurgeStalePeerQueue
        );
        assert_eq!(
            decision.reason_code(),
            "runtime_backpressure_purge_stale_peer_queue"
        );
    }

    #[test]
    fn functional_runtime_queue_enforces_reject_action_on_enqueue() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let mut queue = BoundedRuntimeQueue::new(10).expect("queue should build");
        for index in 0..9 {
            queue
                .enqueue(format!("evt-{index}"))
                .expect("preload should stay in bounds");
        }

        let error = queue
            .enqueue_with_backpressure(
                "evt-reject".to_owned(),
                &controller,
                "kamn:did:agent:peer-bp",
                PeerLifecycleState::Active,
            )
            .expect_err("enqueue should be rejected at saturation threshold");
        assert_eq!(
            error,
            RuntimeQueueError::BackpressureRejected {
                reason_code: "runtime_backpressure_reject_new_enqueue",
                queue_utilization_per_mille: 900,
            }
        );
        assert_eq!(
            error.reason_code(),
            "runtime_backpressure_reject_new_enqueue"
        );
        assert_eq!(queue.len(), 9);
    }

    #[test]
    fn integration_runtime_queue_enforces_stale_peer_purge_action() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let mut queue = BoundedRuntimeQueue::new(8).expect("queue should build");
        queue
            .enqueue("evt-1".to_owned())
            .expect("preload should succeed");
        queue
            .enqueue("evt-2".to_owned())
            .expect("preload should succeed");

        let error = queue
            .enqueue_with_backpressure(
                "evt-disconnected".to_owned(),
                &controller,
                "kamn:did:agent:peer-stale",
                PeerLifecycleState::Disconnected,
            )
            .expect_err("disconnected stale queue should be purged");
        assert_eq!(
            error,
            RuntimeQueueError::BackpressurePurgedStalePeerQueue {
                reason_code: "runtime_backpressure_purge_stale_peer_queue",
                purged_entries: 2,
            }
        );
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn regression_runtime_queue_backpressure_reason_markers_remain_stable() {
        // Regression: #2691
        let decision = RuntimeBackpressureDecision {
            action: RuntimeBackpressureAction::SlowProducer,
            queue_utilization_per_mille: 750,
            stale_peer_queue: false,
        };
        assert_eq!(decision.reason_code(), "runtime_backpressure_slow_producer");

        let queue_error = RuntimeQueueError::BackpressureRejected {
            reason_code: "runtime_backpressure_reject_new_enqueue",
            queue_utilization_per_mille: 950,
        };
        assert_eq!(
            queue_error.reason_code(),
            "runtime_backpressure_reject_new_enqueue"
        );
    }

    #[test]
    fn performance_runtime_queue_backpressure_enforcement_stays_within_ci_budget() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let started = Instant::now();
        for sample_index in 0..2000 {
            let mut queue = BoundedRuntimeQueue::new(16).expect("queue should build");
            let preload = sample_index % 16;
            for event_index in 0..preload {
                queue
                    .enqueue(format!("evt-preload-{event_index}"))
                    .expect("preload should stay bounded");
            }
            let lifecycle_state = if sample_index % 13 == 0 {
                PeerLifecycleState::Disconnected
            } else {
                PeerLifecycleState::Active
            };
            let _ = queue.enqueue_with_backpressure(
                "evt-runtime".to_owned(),
                &controller,
                "kamn:did:agent:peer-perf-runtime",
                lifecycle_state,
            );
        }

        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "runtime queue backpressure enforcement exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    fn regression_runtime_backpressure_rejects_capacity_overflow_sample() {
        // Regression: #618
        assert_eq!(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-a",
                11,
                10,
                PeerLifecycleState::Active
            ),
            Err(RuntimeBackpressureError::QueueDepthExceedsCapacity {
                depth: 11,
                capacity: 10
            })
        );
    }

    #[test]
    fn performance_runtime_backpressure_evaluation_stays_within_ci_budget() {
        let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
        let controller = DeterministicBackpressureController::new(policy);
        let started = Instant::now();
        for sample_index in 0..2000 {
            let queue_depth = (sample_index % 10) + 1;
            let state = if sample_index % 7 == 0 {
                PeerLifecycleState::Disconnected
            } else {
                PeerLifecycleState::Active
            };
            let input =
                RuntimeBackpressureInput::new("kamn:did:agent:peer-perf", queue_depth, 10, state)
                    .expect("input should be valid");
            let _ = controller
                .evaluate(input)
                .expect("evaluation should remain bounded");
        }
        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 200,
            "runtime backpressure evaluation exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    fn unit_authenticated_peer_frame_rejects_invalid_wire_format() {
        assert_eq!(
            AuthenticatedPeerFrame::from_wire("frame|broken"),
            Err(AuthenticatedPeerFrameError::InvalidWireFormat(
                "frame|broken".to_owned()
            ))
        );
    }

    #[test]
    fn functional_authenticated_peer_frame_roundtrips_wire_and_signature() {
        let frame = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("signed frame should build");
        let wire = frame.to_wire().expect("wire encode should pass");
        let decoded = AuthenticatedPeerFrame::from_wire(&wire).expect("wire decode should pass");
        decoded
            .verify_signature()
            .expect("signature verification should pass");
        assert_eq!(decoded, frame);
    }

    #[test]
    fn integration_peer_frame_authenticator_accepts_monotonic_nonce_flow() {
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let frame_1 = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("frame 1 should build");
        let frame_2 = AuthenticatedPeerFrame::signed(
            "frame-2",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            2,
            "payload-2",
        )
        .expect("frame 2 should build");

        assert!(authenticator.validate_inbound(&frame_1).is_ok());
        assert!(authenticator.validate_inbound(&frame_2).is_ok());
    }

    #[test]
    fn regression_forged_or_unauthorized_peer_frame_is_rejected() {
        // Regression: #618
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let forged = AuthenticatedPeerFrame::new(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
            "tampered-signature",
        )
        .expect("frame should build");
        assert!(matches!(
            authenticator.validate_inbound(&forged),
            Err(AuthenticatedPeerFrameError::SignatureMismatch { .. })
        ));

        let unauthorized = AuthenticatedPeerFrame::signed(
            "frame-2",
            "kamn:did:agent:peer-z",
            "kamn:did:agent:peer-b",
            1,
            "payload-2",
        )
        .expect("frame should build");
        assert_eq!(
            authenticator.validate_inbound(&unauthorized),
            Err(AuthenticatedPeerFrameError::UnauthorizedSender(
                "kamn:did:agent:peer-z".to_owned()
            ))
        );
    }

    #[test]
    fn regression_replayed_peer_frame_nonce_is_rejected() {
        // Regression: #618
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let frame = AuthenticatedPeerFrame::signed(
            "frame-1",
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            1,
            "payload-1",
        )
        .expect("frame should build");
        authenticator
            .validate_inbound(&frame)
            .expect("first frame should be accepted");
        assert_eq!(
            authenticator.validate_inbound(&frame),
            Err(AuthenticatedPeerFrameError::ReplayNonce {
                sender_did: "kamn:did:agent:peer-a".to_owned(),
                last_nonce: 1,
                found: 1
            })
        );
    }

    #[test]
    fn performance_authenticated_peer_frame_validation_stays_within_ci_budget() {
        let mut authenticator = PeerFrameAuthenticator::new(
            "kamn:did:agent:peer-b",
            vec!["kamn:did:agent:peer-a".to_owned()],
        )
        .expect("authenticator should build");
        let started = Instant::now();
        for nonce in 1..=256 {
            let frame = AuthenticatedPeerFrame::signed(
                &format!("frame-{nonce}"),
                "kamn:did:agent:peer-a",
                "kamn:did:agent:peer-b",
                nonce,
                "payload-bounded",
            )
            .expect("frame should build");
            authenticator
                .validate_inbound(&frame)
                .expect("frame should be accepted");
        }
        let elapsed_millis = started.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "authenticated peer frame validation exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    fn functional_planner_orders_candidates_deterministically() {
        let candidates = vec![
            ProposalCandidate::new("tx-3", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
            ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            ProposalCandidate::new("tx-2", "did:kamn:agent:bbb", 1, "state-1").expect("valid"),
        ];

        let planner = DeterministicProposalPlanner::new("state-1");
        let plan = planner.plan(candidates).expect("plan should build");
        assert_eq!(
            plan.ordered_candidate_ids(),
            vec!["tx-1".to_owned(), "tx-2".to_owned(), "tx-3".to_owned()]
        );
    }

    #[test]
    fn integration_queue_drains_into_planner_without_order_loss() {
        let mut queue = BoundedRuntimeQueue::new(3).expect("queue should build");
        assert!(queue
            .enqueue(
                ProposalCandidate::new("tx-3", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
            )
            .is_ok());
        assert!(queue
            .enqueue(
                ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            )
            .is_ok());
        assert!(queue
            .enqueue(
                ProposalCandidate::new("tx-2", "did:kamn:agent:bbb", 1, "state-1").expect("valid"),
            )
            .is_ok());

        let mut drained = Vec::new();
        while let Some(candidate) = queue.dequeue() {
            drained.push(candidate);
        }

        let planner = DeterministicProposalPlanner::new("state-1");
        let plan = planner.plan(drained).expect("plan should build");
        assert_eq!(
            plan.ordered_candidate_ids(),
            vec!["tx-1".to_owned(), "tx-2".to_owned(), "tx-3".to_owned()]
        );
    }

    #[test]
    fn unit_rejects_empty_candidate_id() {
        let candidate = ProposalCandidate::new("", "did:kamn:agent:aaa", 1, "state-1");
        assert_eq!(candidate, Err(ProposalPlannerError::InvalidCandidateId));
    }

    #[test]
    fn regression_duplicate_candidate_id_is_rejected() {
        // Regression: #323
        let candidates = vec![
            ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            ProposalCandidate::new("tx-1", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
        ];
        let planner = DeterministicProposalPlanner::new("state-1");
        let error = planner
            .plan(candidates)
            .expect_err("duplicate candidate id must fail");
        assert_eq!(
            error,
            ProposalPlannerError::DuplicateCandidateId("tx-1".to_owned())
        );
    }

    #[test]
    fn regression_stale_state_hash_is_rejected() {
        // Regression: #323
        let candidates = vec![
            ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-2").expect("valid"),
        ];
        let planner = DeterministicProposalPlanner::new("state-1");
        let error = planner
            .plan(candidates)
            .expect_err("candidate state mismatch must fail");
        assert_eq!(
            error,
            ProposalPlannerError::StaleStateHash {
                expected: "state-1".to_owned(),
                found: "state-2".to_owned()
            }
        );
    }

    #[test]
    fn functional_rejoin_guard_accepts_matching_snapshot() {
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let attempt = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
        let status = guard.evaluate(attempt).expect("rejoin should be accepted");
        assert_eq!(status, RecoveryStatus::RejoinAccepted);
    }

    #[test]
    fn integration_rejoin_guard_emits_catch_up_required_for_lagging_node() {
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let attempt = RejoinAttempt::new("node-a", 40, "state-40", "resume-1").expect("valid");
        let status = guard
            .evaluate(attempt)
            .expect("lagging node should receive catch-up guidance");
        assert_eq!(
            status,
            RecoveryStatus::CatchUpRequired {
                from_version: 40,
                to_version: 42
            }
        );
    }

    #[test]
    fn unit_rejoin_guard_rejects_empty_resume_token() {
        let attempt = RejoinAttempt::new("node-a", 42, "state-42", "");
        assert_eq!(attempt, Err(RecoveryGuardError::InvalidResumeToken));
    }

    #[test]
    fn regression_rejoin_replay_token_is_rejected() {
        // Regression: #322
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let first = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
        assert_eq!(guard.evaluate(first), Ok(RecoveryStatus::RejoinAccepted));

        let replay = RejoinAttempt::new("node-a", 42, "state-42", "resume-1").expect("valid");
        let error = guard
            .evaluate(replay)
            .expect_err("replay token should be rejected");
        assert_eq!(
            error,
            RecoveryGuardError::ReplayResumeToken("resume-1".to_owned())
        );
    }

    #[test]
    fn regression_rejoin_state_hash_mismatch_is_rejected() {
        // Regression: #322
        let mut guard = RecoveryRejoinGuard::new(42, "state-42").expect("guard should build");
        let attempt = RejoinAttempt::new("node-a", 42, "state-41", "resume-1").expect("valid");
        let error = guard
            .evaluate(attempt)
            .expect_err("hash mismatch should be rejected");
        assert_eq!(
            error,
            RecoveryGuardError::StateHashMismatch {
                expected: "state-42".to_owned(),
                found: "state-41".to_owned()
            }
        );
    }

    #[test]
    fn functional_snapshot_restore_guard_accepts_matching_snapshot() {
        let guard =
            SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
        let snapshot = RuntimeSnapshot::new(42, "state-42").expect("snapshot should be valid");
        assert!(guard.validate(snapshot).is_ok());
    }

    #[test]
    fn unit_snapshot_restore_guard_rejects_invalid_state_hash() {
        let snapshot = RuntimeSnapshot::new(42, "");
        assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidStateHash));
    }

    #[test]
    fn regression_snapshot_restore_version_mismatch_is_rejected() {
        // Regression: #361
        let guard =
            SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
        let snapshot = RuntimeSnapshot::new(41, "state-42").expect("snapshot should be valid");
        let error = guard
            .validate(snapshot)
            .expect_err("version mismatch should be rejected");
        assert_eq!(
            error,
            SnapshotRestoreError::StateVersionMismatch {
                expected: 42,
                found: 41
            }
        );
    }

    #[test]
    fn regression_snapshot_restore_hash_mismatch_is_rejected() {
        // Regression: #361
        let guard =
            SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
        let snapshot = RuntimeSnapshot::new(42, "state-41").expect("snapshot should be valid");
        let error = guard
            .validate(snapshot)
            .expect_err("hash mismatch should be rejected");
        assert_eq!(
            error,
            SnapshotRestoreError::StateHashMismatch {
                expected: "state-42".to_owned(),
                found: "state-41".to_owned()
            }
        );
    }

    #[test]
    fn functional_snapshot_restore_guard_with_expected_cursor_accepts_matching_snapshot() {
        let guard = SnapshotRestoreGuard::with_expected_cursor(42, "state-42", 100)
            .expect("restore guard should construct");
        let snapshot =
            RuntimeSnapshot::with_cursor(42, "state-42", 100).expect("snapshot should be valid");
        assert!(guard.validate(snapshot).is_ok());
    }

    #[test]
    fn regression_snapshot_restore_cursor_mismatch_is_rejected() {
        // Regression: #617
        let guard = SnapshotRestoreGuard::with_expected_cursor(42, "state-42", 100)
            .expect("restore guard should construct");
        let snapshot =
            RuntimeSnapshot::with_cursor(42, "state-42", 99).expect("snapshot should be valid");
        let error = guard
            .validate(snapshot)
            .expect_err("cursor mismatch should be rejected");
        assert_eq!(
            error,
            SnapshotRestoreError::CursorMismatch {
                expected: 100,
                found: 99
            }
        );
    }

    #[test]
    fn functional_construct_lock_allows_acquire_then_renew_flow() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let renewed = lock
            .renew("processor-a", lease.fencing_token())
            .expect("lease renewal should succeed");
        assert!(renewed.fencing_token() > lease.fencing_token());
    }

    #[test]
    fn unit_construct_lock_rejects_empty_owner_id() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let error = lock
            .acquire_for("")
            .expect_err("empty owner id must be rejected");
        assert_eq!(error, ConstructLockError::InvalidOwnerId);
    }

    #[test]
    fn regression_split_brain_lock_acquisition_is_rejected() {
        // Regression: #362
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        assert!(lock.acquire_for("processor-a").is_ok());
        let error = lock
            .acquire_for("processor-b")
            .expect_err("second owner acquisition must be rejected");
        assert_eq!(
            error,
            ConstructLockError::LeaseAlreadyHeld {
                owner: "processor-a".to_owned()
            }
        );
    }

    #[test]
    fn regression_stale_lease_renewal_is_rejected() {
        // Regression: #362
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let error = lock
            .renew("processor-a", lease.fencing_token().saturating_sub(1))
            .expect_err("stale fencing token must be rejected");
        assert_eq!(
            error,
            ConstructLockError::StaleFencingToken {
                expected: lease.fencing_token(),
                found: lease.fencing_token().saturating_sub(1)
            }
        );
    }

    #[test]
    fn functional_construct_lock_supports_transfer_then_release_flow() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let transferred = lock
            .transfer("processor-a", "processor-b", lease.fencing_token())
            .expect("lease transfer should succeed");
        assert_eq!(transferred.owner_id(), "processor-b");
        assert!(transferred.fencing_token() > lease.fencing_token());
        assert!(lock
            .validate_execution_lease("processor-b", transferred.fencing_token())
            .is_ok());
        assert!(lock
            .release("processor-b", transferred.fencing_token())
            .is_ok());
    }

    #[test]
    fn unit_construct_lock_rejects_release_for_non_owner() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let error = lock
            .release("processor-b", lease.fencing_token())
            .expect_err("non-owner release must be rejected");
        assert_eq!(
            error,
            ConstructLockError::LeaseOwnerMismatch {
                expected: "processor-a".to_owned(),
                found: "processor-b".to_owned()
            }
        );
    }

    #[test]
    fn integration_daemon_tick_requires_matching_active_lease() {
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        assert_eq!(
            execute_processor_daemon_tick(&lock, "processor-a", lease.fencing_token(), 0),
            Ok(1)
        );
    }

    #[test]
    fn regression_unauthorized_transfer_is_rejected() {
        // Regression: #388
        let mut lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let lease = lock
            .acquire_for("processor-a")
            .expect("initial lease acquisition should succeed");
        let error = lock
            .transfer("processor-b", "processor-c", lease.fencing_token())
            .expect_err("unauthorized transfer must be rejected");
        assert_eq!(
            error,
            ConstructLockError::LeaseOwnerMismatch {
                expected: "processor-a".to_owned(),
                found: "processor-b".to_owned()
            }
        );
    }

    #[test]
    fn regression_daemon_tick_without_lease_is_rejected() {
        // Regression: #388
        let lock = ConstructLockGuard::new(5).expect("construct lock should build");
        let error = execute_processor_daemon_tick(&lock, "processor-a", 1, 0)
            .expect_err("daemon execution without active lease must be rejected");
        assert_eq!(error, ConstructLockError::NoLeaseForExecution);
    }

    #[test]
    fn functional_listener_quorum_accepts_canonical_sufficient_attestations() {
        let mut evaluator =
            ListenerQuorumEvaluator::new(2).expect("listener quorum evaluator should build");
        let input = ListenerQuorumInput::new(
            "bridge-event-1",
            1,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-b", "att-2")
                    .expect("valid attestation"),
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");

        let decision = evaluator
            .evaluate(input)
            .expect("quorum should accept canonical listener attestations");
        assert!(decision.accepted);
        assert_eq!(decision.required_confirmations, 2);
        assert_eq!(decision.confirmed_listeners.len(), 2);
        assert_eq!(
            decision.confirmed_listeners,
            vec![
                "kamn:did:agent:listener-a".to_owned(),
                "kamn:did:agent:listener-b".to_owned()
            ]
        );
    }

    #[test]
    fn unit_listener_quorum_rejects_zero_required_confirmations() {
        let error =
            ListenerQuorumEvaluator::new(0).expect_err("zero quorum threshold must be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::InvalidRequiredConfirmations { required: 0 }
        );
    }

    #[test]
    fn integration_daemon_listener_quorum_rejects_replayed_event_sequence() {
        let mut evaluator =
            ListenerQuorumEvaluator::new(1).expect("listener quorum evaluator should build");
        let first = ListenerQuorumInput::new(
            "bridge-event-1",
            3,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        assert!(super::evaluate_daemon_listener_quorum(&mut evaluator, first).is_ok());

        let replay = ListenerQuorumInput::new(
            "bridge-event-1",
            3,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        let error = super::evaluate_daemon_listener_quorum(&mut evaluator, replay)
            .expect_err("replayed sequence should be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::ReplayedEventSequence {
                event_id: "bridge-event-1".to_owned(),
                previous_sequence: 3,
                received_sequence: 3
            }
        );
    }

    #[test]
    fn regression_duplicate_listener_attestation_replay_is_rejected() {
        // Regression: #371
        let mut evaluator =
            ListenerQuorumEvaluator::new(2).expect("listener quorum evaluator should build");
        let input = ListenerQuorumInput::new(
            "bridge-event-dup",
            1,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        let error = evaluator
            .evaluate(input)
            .expect_err("duplicate listener attestations must be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::DuplicateListenerAttestation {
                listener_did: "kamn:did:agent:listener-a".to_owned()
            }
        );
    }

    #[test]
    fn regression_replayed_listener_event_sequence_is_rejected() {
        // Regression: #371
        let mut evaluator =
            ListenerQuorumEvaluator::new(1).expect("listener quorum evaluator should build");
        let first = ListenerQuorumInput::new(
            "bridge-event-regression",
            7,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        assert!(evaluator.evaluate(first).is_ok());

        let replay = ListenerQuorumInput::new(
            "bridge-event-regression",
            6,
            vec![
                ListenerAttestation::new("kamn:did:agent:listener-a", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid listener quorum input");
        let error = evaluator
            .evaluate(replay)
            .expect_err("stale/replayed sequence must be rejected");
        assert_eq!(
            error,
            ListenerQuorumError::ReplayedEventSequence {
                event_id: "bridge-event-regression".to_owned(),
                previous_sequence: 7,
                received_sequence: 6
            }
        );
    }

    #[test]
    fn functional_approver_quorum_authorizes_outbound_with_threshold_attestations() {
        let evaluator =
            ApproverQuorumEvaluator::new(2).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-1",
            "payload-hash-1",
            vec![
                ApproverAttestation::new("kamn:did:agent:approver-a", "payload-hash-1", "att-1")
                    .expect("valid attestation"),
                ApproverAttestation::new("kamn:did:agent:approver-b", "payload-hash-1", "att-2")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid outbound authorization input");

        let decision = evaluator
            .authorize(input)
            .expect("outbound action should be authorized");
        assert!(decision.authorized);
        assert_eq!(decision.required_approvals, 2);
        assert_eq!(
            decision.approved_by,
            vec![
                "kamn:did:agent:approver-a".to_owned(),
                "kamn:did:agent:approver-b".to_owned()
            ]
        );
    }

    #[test]
    fn unit_approver_quorum_rejects_zero_required_approvals() {
        let error =
            ApproverQuorumEvaluator::new(0).expect_err("zero required approvals must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::InvalidRequiredApprovals { required: 0 }
        );
    }

    #[test]
    fn integration_daemon_outbound_approver_quorum_rejects_under_threshold() {
        let evaluator =
            ApproverQuorumEvaluator::new(2).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-under-threshold",
            "payload-hash-2",
            vec![
                ApproverAttestation::new("kamn:did:agent:approver-a", "payload-hash-2", "att-1")
                    .expect("valid attestation"),
            ],
        )
        .expect("valid outbound authorization input");
        let error = authorize_daemon_outbound_action(&evaluator, input)
            .expect_err("under-threshold approvals must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::InsufficientApprovals {
                required: 2,
                received: 1
            }
        );
    }

    #[test]
    fn regression_malformed_approver_payload_is_rejected() {
        // Regression: #372
        let evaluator =
            ApproverQuorumEvaluator::new(1).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-malformed",
            "payload-hash-expected",
            vec![ApproverAttestation::new(
                "kamn:did:agent:approver-a",
                "payload-hash-tampered",
                "att-1",
            )
            .expect("valid attestation")],
        )
        .expect("valid outbound authorization input");
        let error = evaluator
            .authorize(input)
            .expect_err("payload mismatch must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::PayloadDigestMismatch {
                expected: "payload-hash-expected".to_owned(),
                found: "payload-hash-tampered".to_owned()
            }
        );
    }

    #[test]
    fn regression_outbound_under_quorum_is_rejected() {
        // Regression: #372
        let evaluator =
            ApproverQuorumEvaluator::new(3).expect("approver quorum evaluator should build");
        let input = ApproverQuorumInput::new(
            "outbound-action-regression",
            "payload-hash-regression",
            vec![
                ApproverAttestation::new(
                    "kamn:did:agent:approver-a",
                    "payload-hash-regression",
                    "att-1",
                )
                .expect("valid attestation"),
                ApproverAttestation::new(
                    "kamn:did:agent:approver-b",
                    "payload-hash-regression",
                    "att-2",
                )
                .expect("valid attestation"),
            ],
        )
        .expect("valid outbound authorization input");
        let error = evaluator
            .authorize(input)
            .expect_err("under-threshold approvals must be rejected");
        assert_eq!(
            error,
            ApproverQuorumError::InsufficientApprovals {
                required: 3,
                received: 2
            }
        );
    }

    #[test]
    fn functional_divergence_watchdog_flags_hash_mismatch_as_critical() {
        let evaluator = StateDivergenceEvaluator;
        let input = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            42,
            42,
            "state-hash-expected",
            "state-hash-observed",
            110,
        )
        .expect("valid divergence input");

        let report = evaluator
            .evaluate(input)
            .expect("hash mismatch should emit divergence report");
        assert_eq!(report.status, StateDivergenceStatus::Diverged);
        assert_eq!(report.severity, StateDivergenceSeverity::Critical);
    }

    #[test]
    fn unit_divergence_watchdog_rejects_incomplete_evidence_payload() {
        let error = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            42,
            42,
            "state-hash-expected",
            "",
            110,
        )
        .expect_err("empty observed hash must be rejected");
        assert_eq!(
            error,
            StateDivergenceError::IncompleteEvidenceField {
                field: "observed_state_hash"
            }
        );
    }

    #[test]
    fn integration_daemon_divergence_report_includes_deterministic_evidence_fields() {
        let evaluator = StateDivergenceEvaluator;
        let input = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            42,
            42,
            "state-hash-expected",
            "state-hash-observed",
            110,
        )
        .expect("valid divergence input");

        let report = evaluate_daemon_state_divergence(&evaluator, input)
            .expect("daemon divergence evaluation should succeed");
        assert_eq!(report.evidence.peer_id, "kamn:did:agent:validator-a");
        assert_eq!(report.evidence.expected_state_version, 42);
        assert_eq!(report.evidence.observed_state_version, 42);
        assert_eq!(report.evidence.expected_state_hash, "state-hash-expected");
        assert_eq!(report.evidence.observed_state_hash, "state-hash-observed");
        assert_eq!(report.evidence.observed_at_tick, 110);
        assert_eq!(
            report.incident_fingerprint,
            "state-divergence:kamn:did:agent:validator-a:42:42:state-hash-expected:state-hash-observed"
        );
    }

    #[test]
    fn regression_state_divergence_false_negative_is_rejected() {
        // Regression: #381
        let evaluator = StateDivergenceEvaluator;
        let input = StateDivergenceWatchInput::new(
            "kamn:did:agent:validator-a",
            99,
            99,
            "state-hash-expected",
            "state-hash-mismatched",
            220,
        )
        .expect("valid divergence input");

        let report = evaluate_daemon_state_divergence(&evaluator, input)
            .expect("mismatch must produce divergence report");
        assert_eq!(report.status, StateDivergenceStatus::Diverged);
        assert_ne!(
            report.evidence.expected_state_hash,
            report.evidence.observed_state_hash
        );
    }

    #[test]
    fn functional_watchdog_anomaly_classifies_liveness_degradation_as_warning() {
        let evaluator = WatchdogAnomalyEvaluator;
        let input = WatchdogAnomalyWatchInput::new("sample-liveness", 100, 96, 7, 5, 30, 1)
            .expect("valid anomaly sample");
        let report = evaluator
            .evaluate(input)
            .expect("anomaly classification should succeed");
        assert_eq!(report.kind, WatchdogAnomalyKind::LivenessDegradation);
        assert_eq!(report.severity, WatchdogAnomalySeverity::Warning);
    }

    #[test]
    fn unit_watchdog_anomaly_rejects_invalid_delivery_sample() {
        let error = WatchdogAnomalyWatchInput::new("sample-invalid", 10, 12, 5, 5, 30, 2)
            .expect_err("delivered count above expected must be rejected");
        assert_eq!(
            error,
            WatchdogAnomalyError::InvalidSampleCounts {
                expected_deliveries: 10,
                delivered_deliveries: 12
            }
        );
    }

    #[test]
    fn integration_daemon_watchdog_anomaly_report_includes_summary_fields() {
        let evaluator = WatchdogAnomalyEvaluator;
        let input = WatchdogAnomalyWatchInput::new("sample-censorship", 100, 45, 8, 8, 60, 3)
            .expect("valid anomaly sample");
        let report = evaluate_daemon_watchdog_anomaly(&evaluator, input)
            .expect("daemon anomaly evaluation should succeed");
        assert_eq!(report.sample_id, "sample-censorship");
        assert_eq!(report.kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.severity, WatchdogAnomalySeverity::Critical);
        assert_eq!(report.delivery_ratio_per_mille, 450);
        assert_eq!(report.targeted_peer_count, 3);
        assert_eq!(report.sample_window_secs, 60);
    }

    #[test]
    fn regression_censorship_edge_signal_remains_detected_as_critical() {
        // Regression: #382
        let evaluator = WatchdogAnomalyEvaluator;
        let input = WatchdogAnomalyWatchInput::new("sample-regression", 200, 98, 12, 12, 60, 2)
            .expect("valid anomaly sample");
        let report = evaluate_daemon_watchdog_anomaly(&evaluator, input)
            .expect("edge censorship signal should be classified");
        assert_eq!(report.kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.severity, WatchdogAnomalySeverity::Critical);
    }

    #[test]
    fn unit_network_fault_simulation_rejects_zero_queue_capacity() {
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-invalid",
            "peer-sim-a",
            100,
            99,
            6,
            6,
            30,
            1,
            0,
            2,
        );
        assert_eq!(
            input,
            Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 })
        );
    }

    #[test]
    fn functional_network_fault_simulation_classifies_targeted_packet_loss_as_critical() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-censorship",
            "peer-sim-a",
            100,
            45,
            8,
            8,
            60,
            3,
            8,
            8,
        )
        .expect("valid simulation input");
        let report = simulator
            .simulate(input)
            .expect("simulation should classify targeted packet loss");

        assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Critical);
        assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Active);
        assert_eq!(report.queue_overflow_attempts, 0);
        assert_eq!(
            report.backpressure_last_action,
            RuntimeBackpressureAction::SlowProducer
        );
        assert_eq!(
            report.backpressure_last_reason_code,
            "runtime_backpressure_slow_producer"
        );
    }

    #[test]
    fn integration_daemon_network_fault_simulation_reports_overflow_and_degradation() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-overflow",
            "peer-sim-b",
            120,
            110,
            6,
            4,
            30,
            1,
            2,
            5,
        )
        .expect("valid simulation input");
        let report = super::simulate_daemon_network_fault(&simulator, input)
            .expect("simulation should pass");

        assert_eq!(report.final_lifecycle_state, PeerLifecycleState::Degraded);
        assert_eq!(report.queue_overflow_attempts, 3);
        assert_eq!(
            report.backpressure_last_action,
            RuntimeBackpressureAction::RejectNewEnqueue
        );
        assert_eq!(
            report.backpressure_last_reason_code,
            "runtime_backpressure_reject_new_enqueue"
        );
        assert_eq!(report.backpressure_rejected_events, 3);
        assert_eq!(report.backpressure_purged_events, 0);
        assert_eq!(
            report.watchdog_kind,
            WatchdogAnomalyKind::LivenessDegradation
        );
    }

    #[test]
    fn integration_network_fault_simulation_purges_stale_disconnected_peer_queue() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-stale-peer",
            "peer-sim-stale",
            120,
            118,
            6,
            0,
            30,
            1,
            4,
            4,
        )
        .expect("valid simulation input");
        let report = super::simulate_daemon_network_fault(&simulator, input)
            .expect("simulation should pass");

        assert_eq!(
            report.final_lifecycle_state,
            PeerLifecycleState::Disconnected
        );
        assert_eq!(
            report.backpressure_last_action,
            RuntimeBackpressureAction::PurgeStalePeerQueue
        );
        assert_eq!(
            report.backpressure_last_reason_code,
            "runtime_backpressure_purge_stale_peer_queue"
        );
        assert!(report.backpressure_purged_events > 0);
    }

    #[test]
    fn regression_network_fault_simulation_keeps_censorship_critical_boundary() {
        // Regression: #618
        let simulator = DeterministicNetworkFaultSimulator::default();
        let input = NetworkFaultSimulationInput::new(
            "fault-sample-regression",
            "peer-sim-c",
            200,
            100,
            12,
            12,
            60,
            2,
            4,
            4,
        )
        .expect("valid simulation input");
        let report = simulator
            .simulate(input)
            .expect("simulation should classify censorship boundary");

        assert_eq!(report.watchdog_kind, WatchdogAnomalyKind::CensorshipSignal);
        assert_eq!(report.watchdog_severity, WatchdogAnomalySeverity::Critical);
    }

    #[test]
    fn performance_network_fault_simulation_pr_lane_stays_within_budget() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        let start = Instant::now();
        for sample_index in 0..256 {
            let input = NetworkFaultSimulationInput::new(
                &format!("fault-sample-perf-{sample_index}"),
                "peer-sim-perf",
                100,
                98,
                16,
                16,
                30,
                1,
                8,
                8,
            )
            .expect("valid simulation input");
            assert!(simulator.simulate(input).is_ok());
        }
        let elapsed_millis = start.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "network fault simulation PR lane exceeded budget: {elapsed_millis}ms"
        );
    }

    #[test]
    #[ignore = "scheduled chaos lane"]
    fn performance_network_fault_simulation_chaos_lane_stress() {
        let simulator = DeterministicNetworkFaultSimulator::default();
        for sample_index in 0..5000 {
            let targeted_peer_count = if sample_index % 4 == 0 { 3 } else { 1 };
            let delivered = if targeted_peer_count == 3 { 45 } else { 99 };
            let healthy_peers = if sample_index % 3 == 0 { 8 } else { 10 };
            let input = NetworkFaultSimulationInput::new(
                &format!("fault-sample-chaos-{sample_index}"),
                "peer-sim-chaos",
                100,
                delivered,
                10,
                healthy_peers,
                30,
                targeted_peer_count,
                16,
                24,
            )
            .expect("valid simulation input");
            assert!(simulator.simulate(input).is_ok());
        }
    }

    #[test]
    fn functional_in_memory_snapshot_store_round_trips_snapshots() {
        let mut store = InMemoryRuntimeSnapshotStore::default();
        assert!(store.list().expect("list should succeed").is_empty());

        let snapshot_1 = RuntimeSnapshot::new(41, "state-41").expect("valid snapshot");
        let snapshot_2 = RuntimeSnapshot::new(42, "state-42").expect("valid snapshot");
        assert!(store.write(snapshot_1).is_ok());
        assert!(store.write(snapshot_2.clone()).is_ok());

        let latest = store.read_latest().expect("read_latest should succeed");
        assert_eq!(latest, Some(snapshot_2));
        assert_eq!(store.list().expect("list should succeed").len(), 2);
    }

    #[test]
    fn integration_file_snapshot_store_round_trips_snapshots() {
        let path = temp_snapshot_store_path("roundtrip");
        let _ = fs::remove_file(&path);

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let snapshot_1 = RuntimeSnapshot::new(41, "state-41").expect("valid snapshot");
        let snapshot_2 = RuntimeSnapshot::new(42, "state-42").expect("valid snapshot");
        assert!(store.write(snapshot_1).is_ok());
        assert!(store.write(snapshot_2.clone()).is_ok());

        let latest = store.read_latest().expect("read_latest should succeed");
        assert_eq!(latest, Some(snapshot_2));
        assert_eq!(store.list().expect("list should succeed").len(), 2);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_rejects_malformed_payload() {
        // Regression: #387
        let path = temp_snapshot_store_path("malformed");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "not-a-valid-snapshot-line\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let error = store
            .list()
            .expect_err("malformed payload must be rejected");
        assert_eq!(
            error,
            SnapshotStoreError::InvalidPayload("not-a-valid-snapshot-line".to_owned())
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn unit_file_snapshot_store_recovery_handles_missing_snapshot_file() {
        let path = temp_snapshot_store_path("recover-missing");
        let _ = fs::remove_file(&path);

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert!(result.latest.is_none());
        assert_eq!(result.recovered_entries, 0);
        assert_eq!(result.dropped_corrupt_entries, 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_snapshot_store_recovery_recovers_latest_after_trailing_corruption() {
        let path = temp_snapshot_store_path("recover-trailing-corruption");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41\n42|state-42\n43|\n").is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::InvalidPayload("43|".to_owned()))
        );

        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(
            result.latest,
            Some(RuntimeSnapshot::new(42, "state-42").expect("valid snapshot"))
        );
        assert_eq!(result.recovered_entries, 2);
        assert_eq!(result.dropped_corrupt_entries, 1);
        assert_eq!(store.list().expect("list should succeed").len(), 2);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn integration_file_snapshot_store_recovery_allows_append_after_restart() {
        let path = temp_snapshot_store_path("recover-restart");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41\n42|state-42\n43|\n").is_ok());

        let mut first_store =
            FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let first_recovery = first_store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(first_recovery.recovered_entries, 2);

        let mut restarted_store =
            FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let next_snapshot = RuntimeSnapshot::new(43, "state-43").expect("valid snapshot");
        assert!(restarted_store.write(next_snapshot.clone()).is_ok());
        assert_eq!(
            restarted_store.read_latest().expect("read should pass"),
            Some(next_snapshot)
        );
        assert_eq!(restarted_store.list().expect("list should pass").len(), 3);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_recovery_truncates_corrupt_suffix() {
        // Regression: #617
        let path = temp_snapshot_store_path("recover-corrupt-suffix");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41\n42|state-42\nbroken\n43|state-43\n").is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(
            result.latest,
            Some(RuntimeSnapshot::new(42, "state-42").expect("valid snapshot"))
        );
        assert_eq!(result.recovered_entries, 2);
        assert_eq!(result.dropped_corrupt_entries, 2);
        assert_eq!(
            fs::read_to_string(&path).expect("snapshot file should be readable"),
            "41|state-41|41\n42|state-42|42\n"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn unit_runtime_snapshot_with_cursor_rejects_zero_cursor() {
        let snapshot = RuntimeSnapshot::with_cursor(42, "state-42", 0);
        assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidCursor));
    }

    #[test]
    fn unit_runtime_snapshot_rejects_hash_with_metadata_delimiter() {
        let snapshot = RuntimeSnapshot::new(42, "state-42|100");
        assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidStateHash));
    }

    #[test]
    fn unit_in_memory_snapshot_store_rejects_state_version_regression() {
        let mut store = InMemoryRuntimeSnapshotStore::default();
        let baseline = RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot");
        assert!(store.write(baseline).is_ok());
        let stale = RuntimeSnapshot::with_cursor(40, "state-40", 101).expect("valid snapshot");
        assert_eq!(
            store.write(stale),
            Err(SnapshotStoreError::StateVersionRegression {
                previous: 41,
                found: 40
            })
        );
    }

    #[test]
    fn unit_in_memory_snapshot_store_rejects_cursor_regression() {
        let mut store = InMemoryRuntimeSnapshotStore::default();
        let baseline = RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot");
        assert!(store.write(baseline).is_ok());
        let stale = RuntimeSnapshot::with_cursor(42, "state-42", 99).expect("valid snapshot");
        assert_eq!(
            store.write(stale),
            Err(SnapshotStoreError::CursorRegression {
                previous: 100,
                found: 99
            })
        );
    }

    #[test]
    fn regression_file_snapshot_store_rejects_version_regression_metadata() {
        // Regression: #617
        let path = temp_snapshot_store_path("version-regression");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n40|state-40|101\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::StateVersionRegression {
                previous: 41,
                found: 40
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_rejects_cursor_regression_metadata() {
        // Regression: #617
        let path = temp_snapshot_store_path("cursor-regression");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n42|state-42|99\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::CursorRegression {
                previous: 100,
                found: 99
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_snapshot_store_rejects_stale_hash_metadata() {
        // Regression: #617
        let path = temp_snapshot_store_path("hash-regression");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n42|state-41|101\n").is_ok());

        let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        assert_eq!(
            store.list(),
            Err(SnapshotStoreError::StaleStateHash {
                state_hash: "state-41".to_owned(),
                previous_version: 41,
                found_version: 42
            })
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_snapshot_store_recovery_truncates_stale_metadata_suffix() {
        let path = temp_snapshot_store_path("recover-stale-metadata");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "41|state-41|100\n42|state-42|99\n43|state-43|102\n").is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert_eq!(
            result.latest,
            Some(RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot"))
        );
        assert_eq!(result.recovered_entries, 1);
        assert_eq!(result.dropped_corrupt_entries, 2);
        assert_eq!(
            fs::read_to_string(&path).expect("snapshot file should be readable"),
            "41|state-41|100\n"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn performance_file_snapshot_store_recovery_scan_stays_within_ci_budget() {
        let path = temp_snapshot_store_path("recover-performance");
        let _ = fs::remove_file(&path);
        let mut payload = String::new();
        for state_version in 1..=256 {
            payload.push_str(&format!("{state_version}|state-{state_version}\n"));
        }
        payload.push_str("broken\n");
        assert!(fs::write(&path, payload).is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let start = Instant::now();
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        let elapsed_millis = start.elapsed().as_millis();
        assert_eq!(result.recovered_entries, 256);
        assert_eq!(result.dropped_corrupt_entries, 1);
        assert!(
            elapsed_millis < 250,
            "snapshot recovery exceeded CI budget: {elapsed_millis}ms"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    #[ignore = "scheduled snapshot deep lane"]
    fn performance_file_snapshot_store_recovery_deep_lane_large_payload() {
        let path = temp_snapshot_store_path("recover-deep-lane");
        let _ = fs::remove_file(&path);
        let mut payload = String::new();
        for state_version in 1..=8192 {
            payload.push_str(&format!(
                "{state_version}|state-{state_version}|{state_version}\n"
            ));
        }
        payload.push_str("8193|state-8193|0\n");
        assert!(fs::write(&path, payload).is_ok());

        let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
        let start = Instant::now();
        let result = store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        let elapsed_millis = start.elapsed().as_millis();
        assert_eq!(result.recovered_entries, 8192);
        assert_eq!(result.dropped_corrupt_entries, 1);
        assert!(
            elapsed_millis < 2000,
            "snapshot deep-lane recovery exceeded budget: {elapsed_millis}ms"
        );

        let _ = fs::remove_file(path);
    }

    fn temp_snapshot_store_path(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-runtime-snapshot-{tag}-{nonce}.log"))
    }
}
