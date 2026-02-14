use super::{
    authorize_daemon_outbound_action, build_runtime_wiring, evaluate_daemon_state_divergence,
    evaluate_daemon_watchdog_anomaly, execute_processor_daemon_tick, ApproverAttestation,
    ApproverQuorumError, ApproverQuorumEvaluator, ApproverQuorumInput, AuthenticatedPeerFrame,
    AuthenticatedPeerFrameError, BoundedRuntimeQueue, ConstructLockError, ConstructLockGuard,
    DeterministicBackpressureController, DeterministicProposalPlanner, ListenerAttestation,
    ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput, PeerFrameAuthenticator,
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, ProposalPlannerError,
    RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
    RuntimeBackpressureAction, RuntimeBackpressureDecision, RuntimeBackpressureError,
    RuntimeBackpressureInput, RuntimeBackpressurePolicy, RuntimeLifecycleError, RuntimeQueueError,
    StateDivergenceError, StateDivergenceEvaluator, StateDivergenceSeverity, StateDivergenceStatus,
    StateDivergenceWatchInput, WatchdogAnomalyError, WatchdogAnomalyEvaluator, WatchdogAnomalyKind,
    WatchdogAnomalySeverity, WatchdogAnomalyWatchInput,
};
use crate::config::{NodeConfig, NodeRole, SyncMode};
use std::time::Instant;

#[path = "runtime_tests_network_fault.rs"]
mod runtime_tests_network_fault;
#[path = "runtime_tests_snapshot_store.rs"]
mod runtime_tests_snapshot_store;

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
fn regression_runtime_source_routes_tests_via_dedicated_module_file() {
    // Regression: #3192
    let runtime_source = include_str!("runtime.rs");
    let declaration = [
        "#[cfg(test)]",
        "#[path = \"runtime_tests.rs\"]",
        "mod tests;",
    ]
    .join("\n");
    let inline_pattern = ["#[cfg(test)]", "mod tests {"].join("\n");
    assert!(
        runtime_source.contains(&declaration),
        "expected runtime test module declaration to route through runtime_tests.rs"
    );
    assert!(
        !runtime_source.contains(&inline_pattern),
        "expected inline runtime test body to be removed from runtime.rs"
    );
}

#[test]
fn regression_runtime_tests_source_routes_snapshot_store_domain_via_dedicated_module_file() {
    // Regression: #3207
    let runtime_tests_source = include_str!("runtime_tests.rs");
    let declaration = [
        "#[path = \"runtime_tests_snapshot_store.rs\"]",
        "mod runtime_tests_snapshot_store;",
    ]
    .join("\n");
    assert!(
        runtime_tests_source.contains(&declaration),
        "expected runtime tests source to declare dedicated snapshot-store test module"
    );

    for legacy_marker in [
        [
            "fn functional_",
            "in_memory_snapshot_store_round_trips_snapshots()",
        ]
        .concat(),
        [
            "fn integration_file_",
            "snapshot_store_round_trips_snapshots()",
        ]
        .concat(),
        [
            "fn performance_file_snapshot_store_",
            "recovery_scan_stays_within_ci_budget()",
        ]
        .concat(),
    ] {
        assert!(
            !runtime_tests_source.contains(&legacy_marker),
            "expected snapshot-store test `{legacy_marker}` to move out of runtime_tests.rs"
        );
    }
}

#[test]
fn regression_runtime_tests_source_routes_network_fault_domain_via_dedicated_module_file() {
    // Regression: #3212
    let runtime_tests_source = include_str!("runtime_tests.rs");
    let declaration = [
        "#[path = \"runtime_tests_network_fault.rs\"]",
        "mod runtime_tests_network_fault;",
    ]
    .join("\n");
    assert!(
        runtime_tests_source.contains(&declaration),
        "expected runtime tests source to declare dedicated network-fault test module"
    );

    for legacy_marker in [
        [
            "fn unit_network_fault_simulation_",
            "rejects_zero_queue_capacity()",
        ]
        .concat(),
        [
            "fn integration_daemon_network_fault_simulation_",
            "reports_overflow_and_degradation()",
        ]
        .concat(),
        [
            "fn performance_network_fault_simulation_",
            "pr_lane_stays_within_budget()",
        ]
        .concat(),
    ] {
        assert!(
            !runtime_tests_source.contains(&legacy_marker),
            "expected network-fault test `{legacy_marker}` to move out of runtime_tests.rs"
        );
    }
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
    let input =
        RuntimeBackpressureInput::new("kamn:did:agent:peer-a", 8, 10, PeerLifecycleState::Active)
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
        RuntimeBackpressureInput::new("kamn:did:agent:peer-a", 11, 10, PeerLifecycleState::Active),
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
    assert!(
        queue
            .enqueue(
                ProposalCandidate::new("tx-3", "did:kamn:agent:bbb", 2, "state-1").expect("valid"),
            )
            .is_ok()
    );
    assert!(
        queue
            .enqueue(
                ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-1").expect("valid"),
            )
            .is_ok()
    );
    assert!(
        queue
            .enqueue(
                ProposalCandidate::new("tx-2", "did:kamn:agent:bbb", 1, "state-1").expect("valid"),
            )
            .is_ok()
    );

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
    let candidates =
        vec![ProposalCandidate::new("tx-1", "did:kamn:agent:aaa", 1, "state-2").expect("valid")];
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
