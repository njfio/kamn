// Consolidated docs-contract wave 4 harness.
// Migrates low-coupling include_str suites into one file while preserving assertions.

mod transaction_guards_docs {
    const DOC: &str = include_str!("../../../docs/foundation/transaction-guards.md");

    #[test]
    fn doc_contains_transaction_guard_scope_and_components() {
        assert!(DOC.contains("## Invariants Enforced"));
        assert!(DOC.contains("BaselineTransaction"));
        assert!(DOC.contains("TransactionGuards"));
        assert!(DOC.contains("TransactionGuardError"));
    }

    #[test]
    fn doc_contains_canonical_signature_profile_contract() {
        assert!(DOC.contains("## Canonical Signature Profile"));
        assert!(DOC.contains("baseline_signature_for_fields(...)"));
        assert!(DOC.contains("signature_profile_compatibility_fixtures_for_fields(...)"));
        assert!(DOC.contains("legacy-unversioned"));
        assert!(DOC.contains("baseline-v0"));
        assert!(DOC.contains("secp256k1+baseline-v1"));
        assert!(DOC.contains("shared between `transaction` and `signer_backend` paths"));
        assert!(DOC.contains("baseline signature algorithm: `ed25519`."));
        assert!(DOC.contains("baseline signature profile id: `baseline-v1`"));
    }

    #[test]
    fn doc_contains_signer_fallback_policy_integration_rules() {
        assert!(DOC.contains("## Signer Fallback Policy Integration"));
        assert!(DOC.contains("secure:aws-kms:role-<operator|admin|treasury|auditor>/<key-ref>"));
        assert!(DOC.contains("KeyRoleMismatch"));
        assert!(DOC.contains("FallbackDeniedByRolePolicy"));
    }

    #[test]
    fn regression_requires_signature_profile_drift_guard_rule() {
        // Regression: #400
        assert!(DOC.contains(
            "signature-profile drift between transaction and signer paths is rejected (`Regression: #400`).",
        ));
        assert!(DOC.contains("non-versioned signature profile is rejected (`Regression: #404`)."));
        assert!(DOC.contains(
            "algorithm/profile metadata drift or downgrade is rejected (`Regression: #677`)."
        ));
        assert!(DOC.contains(
            "compatibility fixture matrix remains aligned between signer and transaction verification (`Regression: #677`)."
        ));
        assert!(DOC.contains(
            "privileged roles (`admin`, `treasury`, `auditor`) reject fallback via `FallbackDeniedByRolePolicy` (`Regression: #619`).",
        ));
    }
}

mod instruction_verification_docs {
    const DOC: &str = include_str!("../../../docs/foundation/instruction-verification.md");

    #[test]
    fn doc_contains_instruction_verification_scope_and_checks() {
        assert!(DOC.contains("# Instruction Verification Pipeline"));
        assert!(DOC.contains("## Verification Checks"));
        assert!(DOC.contains("InstructionVerifier::verify(...)"));
    }

    #[test]
    fn regression_requires_overlong_claim_window_rejection_rule() {
        // Regression: #409
        assert!(DOC.contains("bounded claim validity window"));
        assert!(DOC.contains("OverlongValidityWindow"));
        assert!(DOC.contains("overlong validity window is rejected (`Regression: #409`)"));
    }

    #[test]
    fn regression_requires_replay_claim_rejection_rule() {
        // Regression: #414
        assert!(DOC.contains("one-time claim consumption"));
        assert!(DOC.contains("ReplayClaim"));
        assert!(DOC.contains("replayed claim is rejected (`Regression: #414`)"));
    }

    #[test]
    fn regression_requires_inclusion_proof_binding_rules() {
        // Regression: #448
        assert!(DOC.contains("inclusion proof reference"));
        assert!(DOC.contains("MissingInclusionProofReference"));
        assert!(DOC.contains("InclusionProofMismatch"));
        assert!(DOC.contains(
            "mismatched or missing inclusion proof reference is rejected (`Regression: #448`)"
        ));
    }

    #[test]
    fn regression_requires_sender_did_validation_rules() {
        // Regression: #453
        assert!(DOC.contains("sender DID format validation"));
        assert!(DOC.contains("InvalidClaimSenderDid"));
        assert!(DOC.contains("InvalidRecordSenderDid"));
        assert!(
            DOC.contains("malformed claim or record sender DID is rejected (`Regression: #453`)")
        );
    }

    #[test]
    fn regression_requires_non_empty_signature_rules() {
        // Regression: #553
        assert!(DOC.contains("Claim and on-chain signatures must be non-empty."));
        assert!(DOC.contains("MissingClaimSignature"));
        assert!(DOC.contains("MissingRecordSignature"));
        assert!(DOC.contains("empty claim or record signatures are rejected (`Regression: #553`)."));
    }
}

mod agent_upgrade_workflow_docs {
    const DOC: &str =
        include_str!("../../../docs/foundation/agent-driven-upgrade-proposal-workflow.md");

    #[test]
    fn doc_contains_agent_upgrade_scope_and_models() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("AgentDrivenUpgradeWorkflow"));
        assert!(DOC.contains("AgentUpgradeWorkflowConfig"));
        assert!(DOC.contains("AgentUpgradeWorkflowError"));
    }

    #[test]
    fn doc_contains_workflow_safeguards_and_audit_rules() {
        assert!(DOC.contains("## Workflow Safeguards"));
        assert!(DOC.contains("## Governance and Audit Rules"));
        assert!(DOC.contains("Governance submission requires:"));
        assert!(DOC.contains(
            "configured minimum activation delay elapsed since governance approval timestamp."
        ));
        assert!(DOC.contains("allowlisted validator voter DID"));
        assert!(DOC.contains("allowlisted validator reviewer DID"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test agent_upgrade_workflow"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_human_review_quorum_gating_rule() {
        // Regression: #235
        assert!(DOC.contains("sufficient unique human reviewer approvals"));
    }

    #[test]
    fn regression_requires_activation_delay_rejection_rule() {
        // Regression: #528
        assert!(DOC
            .contains("early activation before required delay is rejected (`Regression: #528`)."));
    }

    #[test]
    fn regression_requires_unauthorized_validator_vote_rejection_rule() {
        // Regression: #533
        assert!(DOC.contains("unauthorized validator vote is rejected (`Regression: #533`)."));
    }

    #[test]
    fn regression_requires_unauthorized_human_reviewer_rejection_rule() {
        // Regression: #538
        assert!(
            DOC.contains("unauthorized human reviewer approval is rejected (`Regression: #538`).")
        );
    }
}

mod bridge_quorum_runtime_docs {
    const DOC: &str = include_str!("../../../docs/foundation/bridge-quorum-runtime.md");

    #[test]
    fn doc_contains_bridge_quorum_scope_and_models() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("## Listener Quorum Workflow Rules"));
        assert!(DOC.contains("## Approver Quorum Workflow Rules"));
        assert!(DOC.contains("## Ingress Relay Normalization Rules"));
        assert!(DOC.contains("## Replay and Redaction Evidence Lane Rules"));
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("listener attestation"));
        assert!(DOC.contains("approver attestation"));
        assert!(DOC.contains("ApproverQuorumEvaluator"));
        assert!(DOC.contains("authorize_daemon_outbound_action"));
    }

    #[test]
    fn doc_contains_bridge_quorum_fast_lane_commands() {
        assert!(DOC.contains("cargo test -p kamn-core --test bridge_quorum_runtime_docs"));
        assert!(DOC.contains("cargo test -p kamn-core --test runtime_network_docs"));
        assert!(DOC.contains("cargo test -p kamn-core approver_quorum"));
        assert!(DOC.contains("bridge_replay_redaction_contract.py"));
        assert!(DOC.contains("bridge_replay_matrix.sh"));
        assert!(DOC.contains("--suites bridge_adapter,discord_bridge"));
        assert!(DOC.contains("run_bridge_ingress_relay_contract_lane.sh"));
        assert!(DOC.contains("run_bridge_outbound_quorum_contract_lane.sh"));
        assert!(DOC.contains("run_bridge_replay_redaction_contract_lane.sh"));
        assert!(DOC.contains("run_bridge_replay_redaction_deep_lane.sh"));
        assert!(DOC.contains("run_cross_chain_outbound_intent_contract_lane.sh"));
        assert!(DOC.contains("cargo fmt --check"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_listener_and_approver_quorum_guard_rules() {
        // Regression: #373
        assert!(DOC.contains("Duplicate listener attestation replay is rejected."));
        assert!(DOC.contains("Replayed or out-of-order listener event sequences are rejected."));
        assert!(DOC.contains("Outbound under-quorum approval sets are rejected."));
        assert!(DOC.contains("Malformed approver attestation payload is rejected."));
        assert!(DOC.contains("idempotency-key and payload-hash consistency across attempts"));
        assert!(DOC.contains("Duplicate outbound replay requests are rejected"));
        assert!(DOC.contains("unauthorized approver signature-failure rejection"));
        assert!(DOC.contains("Canonical `envelope.id` and `proof.proof_value` remain bound"));
        assert!(DOC.contains("Malformed ingress payloads fail closed"));
        assert!(DOC.contains("required/provided"));
        assert!(DOC.contains("approver DID reason fields"));
        assert!(DOC.contains("kamn.bridge.replay-redaction-evidence.v1"));
        assert!(DOC.contains("Tampered replay/redaction evidence `final_decision` is rejected"));
        assert!(DOC.contains("Regression: #587"));
        assert!(DOC.contains("Regression: #742"));
        assert!(DOC.contains("Regression: #850"));
        assert!(DOC.contains("Regression: #851"));
        assert!(DOC.contains("Regression: #852"));
    }
}

mod performance_target_benchmarking_docs {
    const DOC: &str = include_str!("../../../docs/foundation/performance-target-benchmarking.md");

    #[test]
    fn doc_contains_prd_13_2_thresholds() {
        assert!(DOC.contains("## PRD 13.2 Target Profile"));
        assert!(DOC.contains("Message Latency (p50) | `< 100ms`"));
        assert!(DOC.contains("Message Latency (p99) | `< 500ms`"));
        assert!(DOC.contains("Throughput | `>= 10,000 msg/sec`"));
        assert!(DOC.contains("Availability | `>= 99.9%`"));
    }

    #[test]
    fn doc_contains_deterministic_aggregation_rules() {
        assert!(DOC.contains("## Deterministic Aggregation Rules"));
        assert!(DOC.contains("`latency_p50_ms`: median across benchmark windows."));
        assert!(DOC.contains("`latency_p99_ms`: max across benchmark windows."));
        assert!(DOC.contains("`throughput_tps`: min across benchmark windows."));
        assert!(DOC.contains("`availability_pct`: min across benchmark windows."));
    }

    #[test]
    fn regression_requires_cost_effective_fast_lane_policy() {
        // Regression: #184
        assert!(DOC.contains("## Fast and Cost-Effective Validation Strategy"));
        assert!(DOC.contains("PR gate (fast lane):"));
        assert!(DOC.contains("Deferred deep validation (slow lane):"));
    }

    #[test]
    fn regression_requires_threshold_gate_commands() {
        // Regression: #595
        assert!(DOC.contains("## CI Threshold Gate Contract"));
        assert!(DOC.contains(".ci/performance-targets.env"));
        assert!(DOC.contains("generate_performance_smoke_report.sh --lane smoke"));
        assert!(DOC.contains("check_performance_thresholds.sh --lane deep"));
    }

    #[test]
    fn regression_requires_runtime_invariant_fuzz_concurrency_budget_contract() {
        // Regression: #897
        assert!(DOC.contains("## Runtime Invariant/Fuzz/Concurrency Budget Contract"));
        assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
        assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
        assert!(DOC.contains("KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS=180"));
        assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
    }

    #[test]
    fn regression_requires_dispute_refund_runtime_budget_contract() {
        // Regression: #904
        assert!(DOC.contains(
            "## Dispute/Refund Property and Concurrency Runtime Budget Contract (Issue #904)"
        ));
        assert!(DOC.contains("dispute_refund_transition_contracts"));
        assert!(
            DOC.contains("performance_dispute_refund_property_contract_lane_stays_within_budget")
        );
        assert!(DOC.contains(
            "integration_escrow_dispute_refund_concurrency_replay_is_deterministic_across_rounds"
        ));
        assert!(DOC.contains("Regression: #904"));
    }
}

mod task_swarm_dag_docs {
    const OPERATIONS_DOC: &str = include_str!("../../../docs/foundation/task-operations.md");
    const STATE_MACHINE_DOC: &str = include_str!("../../../docs/foundation/task-state-machine.md");

    #[test]
    fn docs_define_swarm_dag_command_surface() {
        assert!(OPERATIONS_DOC.contains("SwarmTaskDraft"));
        assert!(OPERATIONS_DOC.contains("submit_swarm_tasks(drafts)"));
        assert!(OPERATIONS_DOC.contains("ready_tasks()"));
        assert!(OPERATIONS_DOC.contains("DependencyNotSatisfied"));
    }

    #[test]
    fn docs_define_dependency_aware_transition_gates() {
        assert!(STATE_MACHINE_DOC.contains("## Dependency-Aware Transition Gates"));
        assert!(STATE_MACHINE_DOC.contains("TaskOperationEngine::start_work"));
        assert!(STATE_MACHINE_DOC
            .contains("all declared dependencies must already be in `Completed` state."));
        assert!(STATE_MACHINE_DOC.contains("`InputRequired`"));
        assert!(STATE_MACHINE_DOC.contains("InputRequired -> InProgress | Failed | Cancelled"));
    }

    #[test]
    fn regression_requires_cyclic_and_premature_transition_guards() {
        // Regression: #472
        assert!(OPERATIONS_DOC.contains("Regression: #472"));
        assert!(STATE_MACHINE_DOC.contains("Regression: #472"));
    }

    #[test]
    fn docs_define_bounded_graph_benchmark_lane() {
        assert!(OPERATIONS_DOC.contains("bounded graph benchmark"));
        assert!(OPERATIONS_DOC.contains("cargo test -p kamn-core --test swarm_task_dag"));
    }

    #[test]
    fn docs_define_snapshot_recovery_validation_rules() {
        assert!(OPERATIONS_DOC.contains("export_snapshot()"));
        assert!(OPERATIONS_DOC.contains("restore_snapshot(snapshot)"));
        assert!(OPERATIONS_DOC.contains("schema version mismatch is rejected."));
        assert!(STATE_MACHINE_DOC.contains("Snapshot restore invariants"));
    }

    #[test]
    fn regression_requires_tampered_snapshot_rejection_rule() {
        // Regression: #502
        assert!(OPERATIONS_DOC.contains("Regression: #502"));
        assert!(STATE_MACHINE_DOC.contains("Regression: #502"));
    }

    #[test]
    fn docs_define_snapshot_roundtrip_benchmark_lane() {
        assert!(OPERATIONS_DOC.contains("snapshot roundtrip benchmark"));
        assert!(OPERATIONS_DOC.contains("cargo test -p kamn-core --test task_operation_snapshot"));
    }

    #[test]
    fn regression_requires_input_required_operation_surface() {
        // Regression: #573
        assert!(OPERATIONS_DOC.contains("request_input(task_id, actor, reason)"));
        assert!(OPERATIONS_DOC.contains("emits `InputRequired` notice."));
    }
}

mod bridge_adapter_docs {
    const DOC: &str = include_str!("../../../docs/foundation/bridge-adapter-abstraction.md");

    #[test]
    fn doc_contains_bridge_adapter_core_contracts() {
        assert!(DOC.contains("# Bridge Adapter Abstraction"));
        assert!(DOC.contains("BridgeAdapterEngine"));
        assert!(DOC.contains("process_inbound_to_envelope(...)"));
        assert!(DOC.contains("run_bridge_replay_harness"));
        assert!(DOC.contains("bridge_replay_suites"));
    }

    #[test]
    fn regression_requires_duplicate_inbound_replay_rejection_rule() {
        // Regression: #423
        assert!(DOC.contains("DuplicateInboundMessageId"));
        assert!(DOC.contains("duplicate inbound event is rejected (`Regression: #423`)"));
    }

    #[test]
    fn regression_requires_duplicate_outbound_replay_rejection_rule() {
        // Regression: #433
        assert!(DOC.contains("DuplicateOutboundRequestId"));
        assert!(DOC.contains("duplicate outbound request is rejected (`Regression: #433`)"));
    }

    #[test]
    fn regression_requires_stale_inbound_rejection_rule() {
        // Regression: #546
        assert!(DOC.contains("StaleInboundMessage"));
        assert!(DOC.contains(
            "stale inbound event beyond freshness window is rejected (`Regression: #546`)"
        ));
    }

    #[test]
    fn regression_requires_single_pass_projection_rule() {
        // Regression: #438
        assert!(DOC.contains("single-pass inbound projection"));
        assert!(DOC.contains(
            "first inbound-to-envelope projection does not self-trigger duplicate replay rejection (`Regression: #438`)"
        ));
    }

    #[test]
    fn regression_requires_cross_chain_single_pass_projection_rule() {
        // Regression: #443
        assert!(DOC.contains(
            "cross-chain inbound projection also preserves single-pass replay safety (`Regression: #443`)"
        ));
    }

    #[test]
    fn regression_requires_bridge_fixture_matrix_guard() {
        // Regression: #587
        assert!(DOC.contains("fixtures/bridge_replay/replay_validation_cases.json"));
        assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
        assert!(DOC.contains("signature-failure"));
        assert!(DOC.contains("adapter subset execution"));
        assert!(DOC.contains("Regression: #587"));
    }

    #[test]
    fn doc_contains_credential_redaction_contract_lane() {
        assert!(DOC.contains("## Credentialed Staging + Redaction Contract"));
        assert!(DOC.contains("run_bridge_credential_redaction_check.py"));
        assert!(DOC.contains("run_bridge_credentialed_contract_lane.sh"));
        assert!(DOC.contains("run_bridge_credentialed_deep_lane.sh"));
    }

    #[test]
    fn regression_requires_credential_leakage_guard() {
        // Regression: #621
        assert!(
            DOC.contains("credential leakage and replay gaps remain blocked (`Regression: #621`)")
        );
        assert!(DOC.contains(
            "staged credentialed bridge lane blocks raw secret exposure in logs/artifacts while retaining replay safety (`Regression: #621`)."
        ));
    }
}

mod sdk_parity_fixture_docs {
    const RUST_DOC: &str = include_str!("../../../docs/foundation/rust-sdk-alpha.md");
    const PYTHON_DOC: &str = include_str!("../../../docs/foundation/python-sdk-beta.md");
    const TYPESCRIPT_DOC: &str = include_str!("../../../docs/foundation/typescript-sdk-beta.md");

    #[test]
    fn docs_reference_shared_sdk_parity_fixture_source() {
        assert!(RUST_DOC.contains("fixtures/sdk_parity/register_validation_cases.json"));
        assert!(PYTHON_DOC.contains("fixtures/sdk_parity/register_validation_cases.json"));
        assert!(TYPESCRIPT_DOC.contains("fixtures/sdk_parity/register_validation_cases.json"));
    }

    #[test]
    fn regression_requires_shared_matrix_command_in_all_sdk_docs() {
        // Regression: #583
        assert!(RUST_DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
        assert!(PYTHON_DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
        assert!(TYPESCRIPT_DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
    }

    #[test]
    fn regression_requires_sdk_fixture_snapshot_drift_checker_commands() {
        // Regression: #940
        assert!(RUST_DOC.contains("register_validation_snapshot.json"));
        assert!(PYTHON_DOC.contains("register_validation_snapshot.json"));
        assert!(TYPESCRIPT_DOC.contains("register_validation_snapshot.json"));
        assert!(RUST_DOC.contains("run_example_fixture_drift_contract_lane.sh"));
        assert!(PYTHON_DOC.contains("run_example_fixture_drift_contract_lane.sh"));
        assert!(TYPESCRIPT_DOC.contains("run_example_fixture_drift_contract_lane.sh"));
    }

    #[test]
    fn rust_doc_references_sdk_schema_shared_contract_script() {
        assert!(RUST_DOC.contains("sdk_schema_compatibility_contract.py"));
        assert!(RUST_DOC.contains("live_transport_smoke_parity_policy_contract.py"));
        assert!(RUST_DOC.contains("live_transport_smoke_parity_lane_contract.py"));
        assert!(RUST_DOC.contains("live_transport_smoke_parity_contract_lane_contract.py"));
        assert!(RUST_DOC.contains("live_transport_parity_contract_lane_contract.py"));
        assert!(RUST_DOC.contains("sdk_schema_compatibility_contract_lane_contract.py"));
        assert!(RUST_DOC.contains("example_fixture_drift_contract_lane_contract.py"));
        assert!(RUST_DOC.contains("example_fixture_drift_policy_contract.py"));
        // Regression: #1182
        assert!(RUST_DOC.contains("`Regression: #1182`"));
        // Regression: #1186
        assert!(RUST_DOC.contains("`Regression: #1186`"));
        // Regression: #1190
        assert!(RUST_DOC.contains("`Regression: #1190`"));
        // Regression: #1192
        assert!(RUST_DOC.contains("`Regression: #1192`"));
        // Regression: #1198
        assert!(RUST_DOC.contains("`Regression: #1198`"));
        // Regression: #1202
        assert!(RUST_DOC.contains("`Regression: #1202`"));
        // Regression: #1206
        assert!(RUST_DOC.contains("`Regression: #1206`"));
        assert!(
            RUST_DOC.contains("smoke parity policy checker wrapper remains pinned to the shared contract implementation marker")
        );
        assert!(RUST_DOC.contains(
            "smoke parity lane wrapper remains pinned to the shared contract implementation marker"
        ));
        assert!(RUST_DOC.contains(
            "smoke parity contract lane wrapper remains pinned to the shared contract implementation marker"
        ));
        assert!(RUST_DOC.contains(
            "parity fast-lane wrapper remains pinned to the shared contract implementation marker"
        ));
        assert!(RUST_DOC.contains(
            "sdk schema compatibility contract lane wrapper remains pinned to the shared contract implementation marker"
        ));
        assert!(RUST_DOC.contains(
            "sdk example fixture drift contract lane wrapper remains pinned to the shared contract implementation marker"
        ));
        assert!(RUST_DOC.contains(
            "sdk example fixture drift policy checker wrapper remains pinned to the shared contract implementation marker"
        ));
    }
}

mod zk_message_proofs_docs {
    const DOC: &str = include_str!("../../../docs/foundation/zk-message-proof-design.md");

    #[test]
    fn doc_contains_kolme_constraints_and_design_options() {
        assert!(DOC.contains("## Kolme Constraints That Shape Design"));
        assert!(DOC.contains("single active processor"));
        assert!(DOC.contains("deterministic re-execution"));
        assert!(DOC.contains("## Architecture Options"));
        assert!(DOC.contains("groth16-processor-only"));
        assert!(DOC.contains("plonkish-batched-envelope"));
        assert!(DOC.contains("stark-recursive-watchdog"));
    }

    #[test]
    fn doc_contains_complexity_trust_and_rollout() {
        assert!(DOC.contains("## Complexity and Trust Assumptions"));
        assert!(DOC.contains("trusted setup ceremony"));
        assert!(DOC.contains("watchdog sampling"));
        assert!(DOC.contains("## Recommended Phase 4 Rollout"));
        assert!(DOC.contains("Phase 4.0 - Feasibility harness"));
        assert!(DOC.contains("Phase 4.1 - Processor verification pilot"));
        assert!(DOC.contains("Phase 4.2 - Validator and watchdog expansion"));
    }

    #[test]
    fn doc_contains_fast_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test zk_message_proofs"));
        assert!(DOC.contains("cargo clippy -- -D warnings"));
    }

    #[test]
    fn regression_requires_boundary_inclusive_evaluation_rule() {
        // Regression: #62
        assert!(DOC.contains("threshold checks are inclusive"));
    }

    #[test]
    fn regression_requires_tampered_processor_proof_rejection_rule() {
        // Regression: #509
        assert!(DOC.contains("## Processor Admission Guard Contract"));
        assert!(DOC.contains("tampered processor proof artifacts are rejected"));
    }

    #[test]
    fn regression_requires_validator_watchdog_mismatch_projection_rule() {
        // Regression: #509
        assert!(DOC.contains("## Validator Quorum and Watchdog Projection Contract"));
        assert!(DOC.contains("ValidatorProofConsensusDecision"));
        assert!(DOC.contains("validator DID output is lexicographically ordered"));
        assert!(DOC.contains("ConsensusValid"));
        assert!(DOC.contains("validator-mismatch"));
        assert!(DOC.contains(
            "invalid-proof mismatch propagation must project as a critical validator mismatch signal"
        ));
    }

    #[test]
    fn regression_requires_witness_artifact_contract_lane_marker() {
        // Regression: #993
        assert!(DOC.contains("## Witness and Artifact Schema Contract Lane"));
        assert!(DOC.contains("run_processor_proof_artifact_contract_lane.sh"));
        assert!(
            DOC.contains("private field selector syntax drift is rejected (`Regression: #993`)")
        );
    }

    #[test]
    fn regression_requires_witness_mutation_fast_and_deep_lane_markers() {
        // Regression: #994
        assert!(DOC.contains("## Witness Mutation Property and Fuzz Lanes"));
        assert!(DOC.contains("run_zk_witness_mutation_contract_lane.sh"));
        assert!(DOC.contains("run_zk_witness_mutation_deep_lane.sh"));
        assert!(DOC.contains("performance_zk_witness_mutation_deep_lane_stress -- --ignored"));
    }

    #[test]
    fn regression_requires_processor_admission_runtime_lane_markers() {
        // Regression: #995
        assert!(DOC.contains("## Processor Admission Runtime Contract Lane"));
        assert!(DOC.contains("run_processor_proof_admission_contract_lane.sh"));
        assert!(DOC.contains(
            "processor proof admission reason signatures remain fail-closed (`Regression: #995`)"
        ));
    }
}

mod kolme_runtime_commit_extraction_plan_docs {
    const DOC: &str =
        include_str!("../../../docs/planning/kolme_runtime_commit_extraction_plan.md");

    #[test]
    fn doc_contains_scope_boundaries_and_target_modules() {
        assert!(DOC.contains("# Kolme Runtime Commit Extraction Plan"));
        assert!(DOC.contains("## Scope Boundary"));
        assert!(DOC.contains("## Target Module Boundaries"));
        assert!(DOC.contains("`kamn-kolme`"));
    }

    #[test]
    fn regression_requires_phase_gates_and_validation_matrix_markers() {
        assert!(DOC.contains("## Phase 1 - Transport and endpoint parsing extraction"));
        assert!(DOC.contains("## Phase 2 - Finality and block-fallback extraction"));
        assert!(DOC.contains("## Phase 3 - Adapter and lifecycle orchestration extraction"));
        assert!(DOC.contains("## Phase 1 Progress"));
        assert!(DOC.contains("## Phase 2 Progress"));
        assert!(DOC.contains("#1820"));
        assert!(DOC.contains("#1826"));
        assert!(DOC.contains("#1836"));
        assert!(DOC.contains("#1838"));
        assert!(DOC.contains("#1840"));
        assert!(DOC.contains("#1842"));
        assert!(DOC.contains("#1844"));
        assert!(DOC.contains("#1846"));
        assert!(DOC.contains("#1848"));
        assert!(DOC.contains("#1850"));
        assert!(DOC.contains("#1852"));
        assert!(DOC.contains("#1854"));
        assert!(DOC.contains("#1856"));
        assert!(DOC.contains("#1858"));
        assert!(DOC.contains("#1860"));
        assert!(DOC.contains("#1862"));
        assert!(DOC.contains("#1864"));
        assert!(DOC.contains("#1866"));
        assert!(DOC.contains("#1868"));
        assert!(DOC.contains("#1870"));
        assert!(DOC.contains("#1872"));
        assert!(DOC.contains("#1874"));
        assert!(DOC.contains("#1876"));
        assert!(DOC.contains("#1878"));
        assert!(DOC.contains("#1880"));
        assert!(DOC.contains("#1882"));
        assert!(DOC.contains("#1884"));
        assert!(DOC.contains("#1886"));
        assert!(DOC.contains("#1888"));
        assert!(DOC.contains("#1890"));
        assert!(DOC.contains("#1892"));
        assert!(DOC.contains("#1894"));
        assert!(DOC.contains("#1896"));
        assert!(DOC.contains("#1898"));
        assert!(DOC.contains("#1900"));
        assert!(DOC.contains("#1902"));
        assert!(DOC.contains("#1904"));
        assert!(DOC.contains("#1906"));
        assert!(DOC.contains("#1908"));
        assert!(DOC.contains("#1910"));
        assert!(DOC.contains("#1912"));
        assert!(DOC.contains("#1914"));
        assert!(DOC.contains("#1916"));
        assert!(DOC.contains("#1918"));
        assert!(DOC.contains("#1920"));
        assert!(DOC.contains("#1922"));
        assert!(DOC.contains("#1924"));
        assert!(DOC.contains("#1926"));
        assert!(DOC.contains("#1928"));
        assert!(DOC.contains("#1930"));
        assert!(DOC.contains("#1932"));
        assert!(DOC.contains("#1934"));
        assert!(DOC.contains("#1936"));
        assert!(DOC.contains("#1938"));
        assert!(DOC.contains("#1940"));
        assert!(DOC.contains("#1942"));
        assert!(DOC.contains("#1944"));
        assert!(DOC.contains("#1946"));
        assert!(DOC.contains("#1948"));
        assert!(DOC.contains("#1950"));
        assert!(DOC.contains("#1952"));
        assert!(DOC.contains("#1954"));
        assert!(DOC.contains("#1956"));
        assert!(DOC.contains("#1958"));
        assert!(DOC.contains("#1960"));
        assert!(DOC.contains("#1962"));
        assert!(DOC.contains("#1964"));
        assert!(DOC.contains("## Validation Matrix"));
        assert!(DOC.contains("Regression: #1814"));
    }
}

mod block_pipeline_docs {
    const DOC: &str = include_str!("../../../docs/architecture/block-pipeline.md");
    const ROADMAP: &str =
        include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

    #[test]
    fn architecture_doc_contains_block_pipeline_core_components() {
        assert!(DOC.contains("MempoolBlockPipeline"));
        assert!(DOC.contains("BlockConsensusRoundInput"));
        assert!(DOC.contains("BlockPipelineCommitReport"));
        assert!(DOC.contains("BlockPipelineError"));
        assert!(DOC.contains("SqliteCanonicalCommitStore"));
        assert!(DOC.contains("build_transport_convergence_evidence_bundle(...)"));
        assert!(DOC.contains("UdpPeerLifecycleTransport"));
    }

    #[test]
    fn architecture_doc_contains_consensus_and_runtime_wiring_contracts() {
        assert!(DOC.contains("ListenerQuorumEvaluator"));
        assert!(DOC.contains("ApproverQuorumEvaluator"));
        assert!(DOC.contains("RoleSmokeNetwork::produce_block"));
        assert!(DOC.contains("consensus-validator"));
    }

    #[test]
    fn roadmap_references_phase_32_initial_block_pipeline_slice() {
        assert!(ROADMAP.contains("Phase 3.2 initial slice delivered"));
        assert!(ROADMAP.contains("Task #2926, Subtask #2927"));
        assert!(ROADMAP.contains("docs/architecture/block-pipeline.md"));
    }

    #[test]
    fn docs_reference_phase_32_live_validation_lane_commands() {
        assert!(DOC.contains("scripts/runtime/validate_block_pipeline_live.sh"));
        assert!(DOC.contains("scripts/runtime/test_validate_block_pipeline_live.sh"));
        assert!(ROADMAP.contains("Phase 3.2 live validation delivered"));
        assert!(ROADMAP.contains("Task #2928, Subtask #2929"));
    }

    #[test]
    fn roadmap_tracks_block_pipeline_live_validation_markers() {
        assert!(ROADMAP.contains("block_pipeline_contract_status=verified"));
        assert!(ROADMAP.contains("docs_contract_status=verified"));
        assert!(ROADMAP.contains("fail_closed_status=verified"));
        assert!(ROADMAP.contains("performance_budget_status=verified"));
    }

    #[test]
    fn regression_doc_tracks_digest_mismatch_fail_closed_guard() {
        // Regression: #2927
        assert!(DOC.contains("Regression: #2927"));
        assert!(DOC.contains("fail_closed_reason_code=block_pipeline_payload_digest_mismatch"));
    }

    #[test]
    fn regression_doc_tracks_sqlite_canonical_commit_store_fail_closed_markers() {
        // Regression: #3580
        assert!(DOC.contains("canonical_commit_store_sqlite_schema_mismatch"));
        assert!(DOC.contains("canonical_commit_store_sqlite_payload_not_utf8"));
        assert!(DOC.contains("canonical_commit_store_sqlite_key_height_mismatch"));
    }

    #[test]
    fn regression_doc_tracks_transport_convergence_fault_matrix_markers() {
        // Regression: #3579
        assert!(DOC.contains("transport_convergence_case_id_missing"));
        assert!(DOC.contains("transport_convergence_commit_height_regression"));
        assert!(DOC.contains("block_pipeline_transport_convergence_faults"));
        assert!(DOC.contains("block_pipeline_transport_convergence_live_sockets"));
        assert!(DOC.contains("Regression: #3652"));
        assert!(DOC.contains("Regression: #3670"));
        assert!(DOC.contains("Regression: #4257"));
        assert!(DOC.contains("Regression: #4258"));
        assert!(DOC.contains("Regression: #4259"));
        assert!(DOC.contains("Regression: #4260"));
        assert!(DOC.contains("p2p_transport_live_socket_send_failed"));
        assert!(DOC.contains("validate_libp2p_convergence_process_isolated_live_contract_lane.sh"));
        assert!(
            DOC.contains("check_libp2p_convergence_process_isolated_live_evidence_convergence.sh")
        );
        assert!(DOC.contains("convergence_reason_codes=fork_choice_stale_block_height"));
        assert!(DOC.contains("finality_taxonomy_mapping_status=verified"));
        assert!(DOC.contains("runbook_marker_parity_status=verified"));
        assert!(DOC.contains(
            "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1"
        ));
        assert!(DOC.contains(
            "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
        ));
        assert!(DOC.contains(
            "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1"
        ));
        assert!(DOC.contains(
            "promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation"
        ));
        assert!(DOC.contains("libp2p_finality_evidence_convergence_status=verified"));
        assert!(DOC.contains(
            "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1"
        ));
        assert!(DOC.contains(
            "libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch"
        ));
    }
}

mod a2a_mcp_interop_docs {
    const INTEROP: &str = include_str!("../../../docs/foundation/a2a-mcp-interoperability.md");

    #[test]
    fn interop_spec_contains_message_type_mapping() {
        assert!(INTEROP.contains("## Message Type Mapping"));
        assert!(INTEROP.contains("| KAMN Envelope Header | A2A Concept | MCP Concept | Notes |"));
        assert!(INTEROP
            .contains("| Request | task.invoke | tool_call | Deterministic request dispatch. |"));
        assert!(INTEROP.contains(
            "| Response | task.result | tool_result | Deterministic completion response. |"
        ));
        assert!(INTEROP
            .contains("| Event | event.notify | notification | Non-blocking external signal. |"));
    }

    #[test]
    fn interop_spec_contains_task_lifecycle_mapping() {
        assert!(INTEROP.contains("## Task Lifecycle Mapping"));
        assert!(
            INTEROP.contains("| KAMN Task State | A2A Task State | MCP-Oriented Interpretation |")
        );
        assert!(INTEROP.contains("| Submitted | pending | Awaiting execution slot. |"));
        assert!(INTEROP.contains("| InProgress | running | Active execution in progress. |"));
        assert!(INTEROP.contains("| Completed | succeeded | Terminal success state. |"));
        assert!(INTEROP.contains("| Failed | failed | Terminal failure state. |"));
    }

    #[test]
    fn interop_spec_contains_sdk_examples_and_fallback_rules() {
        assert!(INTEROP.contains("## Deterministic SDK Examples"));
        assert!(INTEROP.contains("Rust SDK mapping example"));
        assert!(INTEROP.contains("Python SDK mapping example"));
        assert!(INTEROP.contains("TypeScript SDK mapping example"));
        assert!(INTEROP.contains("## Limitations and Fallback Behavior"));
        assert!(INTEROP.contains("Unknown external type maps to ManualReview."));
        assert!(
            INTEROP.contains("Lossy mapping paths must emit interoperability warning metadata.")
        );
    }

    #[test]
    fn regression_requires_ambiguous_mapping_manual_review_rule() {
        // Regression: #177
        assert!(INTEROP.contains("Ambiguous mapping decision: ManualReview."));
    }

    #[test]
    fn interop_spec_contains_a2a_mcp_conformance_harness_commands() {
        assert!(INTEROP.contains("## A2A/MCP Conformance Harness Evidence Contract (Issue #893)"));
        assert!(INTEROP.contains("run_a2a_mcp_conformance_harness.py"));
        assert!(INTEROP.contains("a2a_mcp_conformance_policy_contract.py"));
        assert!(INTEROP.contains("check_a2a_mcp_conformance_policy.sh"));
        assert!(INTEROP.contains("run_a2a_mcp_conformance_contract_lane.sh"));
        assert!(INTEROP.contains("a2a_mcp_conformance_reason_codes:GO:v1"));
    }

    #[test]
    fn regression_requires_a2a_mcp_conformance_schema_drift_fail_closed_policy() {
        // Regression: #893
        assert!(INTEROP.contains("Regression: #893"));
    }
}

mod key_lifecycle_audit_trails_docs {
    const DOC: &str = include_str!("../../../docs/foundation/key-lifecycle-audit-trails.md");

    #[test]
    fn doc_contains_audit_record_schema_and_verification_rules() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("KeyLifecycleAuditRecord"));
        assert!(DOC.contains("KeyLifecycle::audit_records()"));
        assert!(DOC.contains("KeyLifecycle::verify_audit_records(...)"));
        assert!(DOC.contains("KeyLifecycleAuditError"));
    }

    #[test]
    fn doc_contains_tamper_evident_chain_requirements() {
        assert!(DOC.contains("## Tamper-Evident Rules"));
        assert!(DOC.contains("genesis marker `GENESIS`"));
        assert!(DOC.contains("Sequence IDs must be contiguous and start at `1`."));
        assert!(DOC.contains("Verification fails when sequence continuity, chain links, or record hashes are inconsistent."));
    }

    #[test]
    fn regression_requires_chain_link_mismatch_detection_rule() {
        // Regression: #158
        assert!(DOC.contains("chain links"));
    }

    #[test]
    fn doc_contains_lifecycle_operator_binding_audit_evidence_contract() {
        assert!(
            DOC.contains("## DID Lifecycle Operator-Binding Audit Evidence Contract (Issue #890)")
        );
        assert!(DOC.contains("lifecycle_operator_binding_contract.py"));
        assert!(DOC.contains("generate_lifecycle_operator_binding_evidence_bundle.sh"));
        assert!(DOC.contains("check_lifecycle_operator_binding_policy.sh"));
        assert!(DOC.contains("run_lifecycle_operator_binding_contract_lane.sh"));
        assert!(DOC.contains("did_lifecycle_operator_binding_reason_codes:GO:v1"));
    }

    #[test]
    fn regression_doc_marks_missing_keys_or_decision_drift_fail_closed_policy() {
        // Regression: #890
        assert!(DOC.contains("Regression: #890"));
    }

    #[test]
    fn doc_contains_secure_provider_key_lifecycle_evidence_contract() {
        assert!(DOC
            .contains("## Secure-Provider Key Rotation/Revocation Evidence Contract (Issue #988)"));
        assert!(DOC.contains("secure_provider_key_lifecycle_contract.py"));
        assert!(DOC.contains("generate_secure_provider_key_lifecycle_evidence_bundle.sh"));
        assert!(DOC.contains("check_secure_provider_key_lifecycle_policy.sh"));
        assert!(DOC.contains("run_secure_provider_key_lifecycle_contract_lane.sh"));
        assert!(DOC.contains("secure_provider_key_lifecycle_reason_codes:GO:v1"));
    }

    #[test]
    fn regression_doc_marks_secure_provider_lifecycle_fail_closed_policy() {
        // Regression: #988
        assert!(DOC.contains("Regression: #988"));
    }
}

mod didcomm_compatibility_profile_docs {
    const PROFILE: &str =
        include_str!("../../../docs/foundation/didcomm-v2-compatibility-profile.md");

    #[test]
    fn profile_contains_field_level_mapping_table() {
        assert!(PROFILE.contains("## Field-Level Mapping"));
        assert!(
            PROFILE.contains("| KAMN Canonical Field | DIDComm v2 Field | Compatibility Rule |")
        );
        assert!(PROFILE.contains("| envelope.id | id | Preserve as-is. |"));
        assert!(PROFILE.contains("| envelope.from | from | Preserve as DID string. |"));
        assert!(PROFILE.contains("| envelope.to[] | to[] | Preserve recipient DID list order. |"));
        assert!(PROFILE.contains("| body.message | body | Serialize as JSON body payload. |"));
    }

    #[test]
    fn profile_contains_crypto_and_key_handling_expectations() {
        assert!(PROFILE.contains("## Crypto and Key Handling Expectations"));
        assert!(PROFILE.contains(
            "Ed25519 verification methods remain authoritative for signature validation."
        ));
        assert!(PROFILE.contains("X25519 key agreement references must map to recipient key IDs."));
        assert!(PROFILE
            .contains("Unsupported algorithm negotiation results in compatibility rejection."));
    }

    #[test]
    fn profile_contains_deterministic_test_vectors() {
        assert!(PROFILE.contains("## Deterministic Compatibility Vectors"));
        assert!(PROFILE
            .contains("Vector-S1: canonical request envelope maps to DIDComm plaintext message."));
        assert!(PROFILE
            .contains("Vector-S2: canonical response envelope maps to DIDComm signed response."));
        assert!(PROFILE.contains("Vector-F1: missing recipient key reference is rejected."));
        assert!(PROFILE.contains("Vector-F2: unsupported attachment mapping is rejected."));
    }

    #[test]
    fn regression_requires_unsupported_attachment_rejection_rule() {
        // Regression: #179
        assert!(PROFILE.contains("Unsupported attachment translation decision: reject."));
    }

    #[test]
    fn profile_contains_didcomm_envelope_replay_contract_lane_commands() {
        assert!(
            PROFILE.contains("## DIDComm Envelope Compatibility Replay Contract Lane (Issue #892)")
        );
        assert!(PROFILE.contains("run_didcomm_envelope_compatibility_replay.py"));
        assert!(PROFILE.contains("didcomm_envelope_compatibility_policy_contract.py"));
        assert!(PROFILE.contains("check_didcomm_envelope_compatibility_policy.sh"));
        assert!(PROFILE.contains("run_didcomm_envelope_compatibility_contract_lane.sh"));
        assert!(PROFILE.contains("didcomm_envelope_compatibility_reason_codes:GO:v1"));
    }

    #[test]
    fn regression_requires_didcomm_envelope_schema_signature_drift_fail_closed_policy() {
        // Regression: #892
        assert!(PROFILE.contains("Regression: #892"));
    }
}

mod task_escrow_suite_discovery_parallel_contract {
    const ROOT_SUITE: &str = include_str!("task_escrow_proptest_invariants.rs");
    const TASK_DOMAIN_SUITE: &str = include_str!("task_escrow_proptest_invariants/task_domain.rs");
    const ESCROW_DOMAIN_SUITE: &str =
        include_str!("task_escrow_proptest_invariants/escrow_domain.rs");
    const SHARED_SUITE: &str = include_str!("task_escrow_proptest_invariants/shared.rs");

    #[test]
    fn regression_task_escrow_suite_discovery_markers_remain_stable() {
        assert!(ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/shared.rs\"]"));
        assert!(ROOT_SUITE.contains("mod shared;"));
        assert!(ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/task_domain.rs\"]"));
        assert!(ROOT_SUITE.contains("mod task_domain;"));
        assert!(
            ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/escrow_domain.rs\"]")
        );
        assert!(ROOT_SUITE.contains("mod escrow_domain;"));

        assert!(TASK_DOMAIN_SUITE
            .contains("fn functional_task_lifecycle_proptest_sequence_invariants_hold"));
        assert!(TASK_DOMAIN_SUITE.contains(
            "fn functional_task_lifecycle_proptest_transition_evidence_is_legal_and_stable"
        ));
        assert!(TASK_DOMAIN_SUITE
            .contains("fn integration_task_lifecycle_proptest_restore_roundtrip_is_stable"));

        assert!(ESCROW_DOMAIN_SUITE
            .contains("fn integration_escrow_proptest_transition_evidence_preserves_invariants"));
        assert!(ESCROW_DOMAIN_SUITE
            .contains("fn integration_escrow_proptest_conserves_amounts_and_status_projections"));
    }

    #[test]
    fn regression_task_escrow_suite_parallel_boundaries_are_bounded_and_isolated() {
        assert!(SHARED_SUITE.contains("pub const TASK_CASES: u32 = 192;"));
        assert!(SHARED_SUITE.contains("pub const ESCROW_CASES: u32 = 192;"));
        assert!(SHARED_SUITE.contains("pub const MAX_SEQUENCE_LEN: usize = 32;"));
        assert!(SHARED_SUITE
            .contains("pub const TASK_SEED_ENV_KEY: &str = \"KAMN_PROPTEST_TASK_ESCROW_SEED\";"));
        assert!(SHARED_SUITE
            .contains("pub const ESCROW_SEED_ENV_KEY: &str = \"KAMN_PROPTEST_ESCROW_SEED\";"));
    }
}

mod task_payment_workflow_docs {
    const DOC: &str = include_str!("../../../docs/foundation/task-payment-workflow.md");

    #[test]
    fn doc_contains_payment_offer_confirm_models_and_workflow_contract() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("PaymentOffer"));
        assert!(DOC.contains("PaymentConfirm"));
        assert!(DOC.contains("TaskPaymentWorkflow"));
        assert!(DOC.contains("TaskPaymentError"));
    }

    #[test]
    fn doc_contains_completion_and_escrow_validation_rules() {
        assert!(DOC.contains("## Deterministic Validation Rules"));
        assert!(DOC.contains("target task in `Completed` state."));
        assert!(DOC.contains("`payer_did` equal to task requester DID."));
        assert!(DOC.contains("`payee_did` equal to task assignee DID."));
        assert!(DOC.contains("offered amount less than or equal to escrow remaining balance."));
        assert!(DOC.contains("single-use confirmation (duplicates are rejected)."));
        assert!(DOC.contains("current_unix >= timeout_unix"));
        assert!(DOC.contains("## Escrow Release Behavior"));
        assert!(DOC.contains("EscrowLifecycle::refund_after_timeout(current_unix, timeout_unix)"));
    }

    #[test]
    fn regression_requires_duplicate_confirm_rejection_rule() {
        // Regression: #216
        assert!(DOC.contains("duplicates are rejected"));
    }

    #[test]
    fn regression_requires_premature_timeout_refund_rejection_rule() {
        // Regression: #542
        assert!(
            DOC.contains("premature timeout refund attempts are rejected (`Regression: #542`).")
        );
    }

    #[test]
    fn regression_requires_participant_binding_rules() {
        // Regression: #558
        assert!(DOC.contains("`payer_did` equal to task requester DID."));
        assert!(DOC.contains("`payee_did` equal to task assignee DID."));
    }

    #[test]
    fn doc_contains_settlement_evidence_schema_and_checker_contract() {
        assert!(DOC.contains("## Settlement Evidence Schema and Policy Checks"));
        assert!(DOC.contains("generate_settlement_reconciliation_evidence_bundle.sh"));
        assert!(DOC.contains("check_settlement_reconciliation_evidence_policy.sh"));
        assert!(DOC.contains("settlement_reconciliation_reason_codes:GO:v1"));
        assert!(DOC.contains("settlement_reconciliation_reason_codes:NO-GO:v1"));
    }

    #[test]
    fn regression_requires_settlement_evidence_schema_drift_fail_closed_rule() {
        // Regression: #906
        assert!(
            DOC.contains("settlement evidence schema drift must fail closed (`Regression: #906`).")
        );
    }
}

mod task_operations_docs {
    const DOC: &str = include_str!("../../../docs/foundation/task-operations.md");

    #[test]
    fn doc_contains_task_operation_core_types_and_commands() {
        assert!(DOC.contains("## Core Types"));
        assert!(DOC.contains("TaskOperationEngine"));
        assert!(DOC.contains("SwarmTaskDraft"));
        assert!(DOC.contains("## Command Behavior"));
        assert!(DOC.contains("submit(task_id, requester, description)"));
        assert!(DOC.contains("request_input(task_id, actor, reason)"));
    }

    #[test]
    fn doc_contains_snapshot_persistence_and_restore_contract_rules() {
        assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
        assert!(DOC.contains("export_snapshot()"));
        assert!(DOC.contains("restore_snapshot(snapshot)"));
        assert!(DOC.contains("TaskOperationSnapshotStore"));
        assert!(DOC.contains("recover_latest_and_repair()"));
        assert!(DOC.contains("TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane_commands() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --lib task_operations::tests::"));
        assert!(DOC.contains("cargo test -p kamn-core --test task_operations"));
        assert!(DOC.contains("cargo test -p kamn-core --test task_operation_snapshot"));
        assert!(DOC.contains("cargo test -p kamn-core --test task_operations_docs"));
        assert!(DOC.contains("bash scripts/task/run_task_operation_snapshot_contract_lane.sh"));
        assert!(DOC.contains(
            "cargo test -p kamn-core --lib task_operations::tests::performance_task_operation_snapshot_store_deep_lane_stress -- --ignored"
        ));
        assert!(DOC.contains("bash scripts/task/run_task_operation_snapshot_deep_lane.sh"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_task_snapshot_restore_guard_rules() {
        // Regression: #617
        assert!(DOC.contains("duplicate task IDs on restore are rejected (`Regression: #617`)"));
        assert!(DOC.contains("malformed snapshot payloads are rejected (`Regression: #617`)"));
        assert!(DOC.contains(
            "dependency-completion tampering remains rejected during restore (`Regression: #502`)"
        ));
    }

    #[test]
    fn doc_contains_federated_delegation_settlement_contract() {
        assert!(DOC.contains("## Federated Delegation Settlement Evidence Contract"));
        assert!(DOC.contains("generate_federated_delegation_settlement_evidence_bundle.sh"));
        assert!(DOC.contains("check_federated_delegation_settlement_policy.sh"));
        assert!(DOC.contains("run_federated_delegation_settlement_contract_lane.sh"));
        assert!(DOC.contains("run_federated_delegation_settlement_deep_lane.sh"));
        assert!(DOC.contains("run_federated_delegation_settlement_matrix.py"));
        assert!(DOC.contains("fixtures/federated_task_delegation/partition_replay_cases.json"));
    }

    #[test]
    fn regression_requires_federated_delegation_settlement_guard_marker() {
        // Regression: #734
        assert!(DOC.contains(
            "settlement reference drift, replay attempts, and tampered final decisions force `NO-GO` (`Regression: #734`)."
        ));
    }
}

mod did_registry_transactions_docs {
    const DOC: &str = include_str!("../../../docs/foundation/did-registry-transactions.md");

    #[test]
    fn doc_contains_retry_classification_contract() {
        assert!(DOC.contains("idempotency_key_for_register"));
        assert!(DOC.contains("submit_registration_via_chain_adapter"));
        assert!(DOC.contains("register_with_retry_guard"));
        assert!(DOC.contains("NewSubmission"));
        assert!(DOC.contains("RetryableInFlight"));
        assert!(DOC.contains("FinalizedNoRetry"));
        assert!(DOC.contains("ConflictNoRetry"));
    }

    #[test]
    fn doc_contains_finality_safety_rules() {
        assert!(DOC.contains("Submitted(receipt)"));
        assert!(DOC.contains("Duplicate(receipt)"));
        assert!(DOC.contains("Rejected { reason }"));
        assert!(DOC.contains("FinalizedNoOp"));
        assert!(DOC.contains("InMemoryDidRegistrationChainAdapter"));
        assert!(DOC.contains("record_register_finality"));
        assert!(DOC.contains("StaleFinalityUpdate"));
        assert!(DOC.contains("ConflictingFinalityUpdate"));
        assert!(DOC.contains("UnknownSubmissionIdempotencyKey"));
    }

    #[test]
    fn regression_doc_marks_stale_duplicate_conflict_guard() {
        // Regression: #678
        assert!(DOC.contains("Regression: #678"));
    }

    #[test]
    fn doc_contains_lifecycle_mutation_transaction_contracts() {
        assert!(DOC.contains("DidLifecycleMutationRequest"));
        assert!(DOC.contains("apply_lifecycle_mutation"));
        assert!(DOC.contains("submit_lifecycle_mutation_via_chain_adapter"));
        assert!(DOC.contains("KolmeDidLifecycleChainAdapter"));
        assert!(DOC.contains("record_lifecycle_finality"));
        assert!(DOC.contains("did_lifecycle_mutation_allowed"));
        assert!(DOC.contains("did_lifecycle_mutation_nonce_replay"));
        assert!(DOC.contains("did_lifecycle_mutation_unauthorized_actor"));
        assert!(DOC.contains("did_chain_adapter_submit_failed"));
    }

    #[test]
    fn regression_doc_marks_lifecycle_mutation_fail_closed_guard() {
        // Regression: #889
        assert!(DOC.contains("Regression: #889"));
    }

    #[test]
    fn regression_doc_marks_lifecycle_chain_submission_conflict_guard() {
        // Regression: #2936
        assert!(DOC.contains("Regression: #2936"));
    }

    #[test]
    fn regression_doc_marks_registration_payload_integrity_and_duplicate_drift_guards() {
        // Regression: #4418
        assert!(DOC.contains("Regression: #4418"));
        assert!(DOC.contains(
            "did_registration_reason_taxonomy_version=kamn.kolme.did-registration-reason-taxonomy.v1"
        ));
        assert!(DOC.contains("did_registration_reason_codes_csv=did_registry_document_did_mismatch,did_registry_submission_key_conflict"));
    }
}

mod kolme_integration_roadmap_docs {
    const ROADMAP: &str = include_str!("../../../docs/planning/kolme-integration-roadmap.md");

    #[test]
    fn roadmap_contains_version_and_runtime_commit_contract_lane_commands() {
        assert!(ROADMAP.contains("validate_version_compatibility.py"));
        assert!(ROADMAP.contains("generate_fork_compatibility_evidence.py"));
        assert!(ROADMAP.contains("check_fork_compatibility_policy.py"));
        assert!(ROADMAP.contains("run_version_compatibility_contract_lane.sh"));
        assert!(ROADMAP.contains("run_runtime_commit_contract_lane.sh"));
        assert!(ROADMAP.contains("docs/foundation/kolme-runtime-commit-client.md"));
        assert!(ROADMAP.contains("check_runtime_commit_replay_policy.py"));
        assert!(ROADMAP.contains("run_runtime_commit_replay_tamper_matrix.py"));
        assert!(ROADMAP.contains("run_runtime_commit_replay_contract_lane.sh"));
        assert!(ROADMAP.contains("run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_runtime_commit_adapter_contract_lane.json --phase contract"));
        assert!(ROADMAP.contains("kolme_runtime_commit_fork_finality_resolver"));
        assert!(ROADMAP.contains("run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json --phase contract"));
        assert!(ROADMAP.contains("unit_runtime_commit_signed_translation_rejects_message_mismatch"));
        assert!(
            ROADMAP.contains("integration_kolme_fork_signed_envelope_submit_maps_txhash_response")
        );
        assert!(ROADMAP.contains("integration_http_transport_fetch_next_nonce_query_and_parse"));
        assert!(ROADMAP
            .contains("integration_http_transport_submit_broadcast_request_put_and_parse_txhash"));
        assert!(ROADMAP.contains(
            "regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response"
        ));
        assert!(ROADMAP.contains("check_nonce_broadcast_parity_policy.py"));
        assert!(ROADMAP.contains("run_nonce_broadcast_parity_matrix.py"));
        assert!(ROADMAP.contains("run_nonce_broadcast_parity_contract_lane.sh"));
        assert!(ROADMAP.contains("run_notifications_consumer_contract_lane.sh"));
        assert!(ROADMAP.contains("run_block_fallback_reconciliation_contract_lane.sh"));
        assert!(ROADMAP.contains("kolme_runtime_commit_notifications"));
        assert!(ROADMAP.contains("kolme_runtime_commit_block_fallback"));
        assert!(ROADMAP.contains("fixtures/kolme_compatibility/fork_compatibility_cases.json"));
        assert!(ROADMAP.contains("fixtures/kolme_compatibility/lane_migration_matrix.json"));
        assert!(ROADMAP.contains("check_lane_migration_matrix_policy.py"));
        assert!(ROADMAP.contains("fixtures/kolme_commit/runtime_commit_request_cases.txt"));
        assert!(ROADMAP.contains("fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json"));
        assert!(ROADMAP.contains("fixtures/kolme_commit/nonce_broadcast_parity_cases.json"));
        assert!(ROADMAP.contains("fixtures/kolme_commit/local_live_api_conformance_matrix.json"));
        assert!(ROADMAP.contains("--cargo-profile portable"));
        assert!(ROADMAP.contains("--matrix-cargo-profile portable"));
    }

    #[test]
    fn roadmap_local_validation_includes_lane_migration_matrix_policy_test() {
        assert!(ROADMAP.contains("bash scripts/kolme/test_check_lane_migration_matrix_policy.sh"));
    }

    #[test]
    fn regression_guards_include_legacy_and_runtime_commit_markers() {
        assert!(ROADMAP.contains("`Regression: #775`"));
        assert!(ROADMAP.contains("`Regression: #825`"));
        assert!(ROADMAP.contains("`Regression: #826`"));
        assert!(ROADMAP.contains("`Regression: #827`"));
        assert!(ROADMAP.contains("`Regression: #979`"));
        assert!(ROADMAP.contains("`Regression: #980`"));
        assert!(ROADMAP.contains("`Regression: #1502`"));
        assert!(ROADMAP.contains("`Regression: #1503`"));
        assert!(ROADMAP.contains("`Regression: #1504`"));
        assert!(ROADMAP.contains("`Regression: #1506`"));
        assert!(ROADMAP.contains("`Regression: #1401`"));
        assert!(ROADMAP.contains("`Regression: #1402`"));
        assert!(ROADMAP.contains("`Regression: #1462`"));
        assert!(ROADMAP.contains("`Regression: #1463`"));
        assert!(ROADMAP.contains("`Regression: #1464`"));
        assert!(ROADMAP.contains("`Regression: #1533`"));
        assert!(ROADMAP.contains("`Regression: #1659`"));
    }
}

mod escrow_lifecycle_docs {
    const DOC: &str = include_str!("../../../docs/foundation/escrow-lifecycle.md");

    #[test]
    fn doc_contains_escrow_lifecycle_scope_and_transitions() {
        assert!(DOC.contains("# Escrow Lifecycle State Machine"));
        assert!(DOC.contains("EscrowLifecycle"));
        assert!(DOC.contains("Disputed -> Resolved"));
    }

    #[test]
    fn doc_contains_settlement_reconciliation_evidence_contract() {
        assert!(DOC.contains("## Settlement Reconciliation Evidence Contract"));
        assert!(DOC.contains("generate_settlement_reconciliation_evidence_bundle.sh"));
        assert!(DOC.contains("check_settlement_reconciliation_evidence_policy.sh"));
        assert!(DOC.contains("run_settlement_reconciliation_contract_lane.sh"));
        assert!(DOC.contains("run_settlement_reconciliation_deep_lane.sh"));
        assert!(DOC.contains("--ledger-reference-id"));
    }

    #[test]
    fn doc_contains_chain_receipt_finality_adapter_contract() {
        assert!(DOC.contains("## Chain Receipt Finality Adapter Contract"));
        assert!(DOC.contains("EscrowReceiptFinality::{Final, Pending, Failed}"));
        assert!(DOC.contains("reconcile_receipt_finality(receipt_id, finality, action)"));
        assert!(DOC.contains("EscrowSettlementOutcome::{Settled, Pending, Rejected}"));
        assert!(DOC.contains("EscrowReceiptFinality::parse(...)"));
    }

    #[test]
    fn doc_contains_transition_evidence_reason_code_contract() {
        assert!(DOC.contains("## Transition Evidence and Reason-Code Contract (Issue #903)"));
        assert!(DOC.contains("apply_transition_with_evidence"));
        assert!(DOC.contains("EscrowTransitionEvidence"));
        assert!(DOC.contains("escrow_transition_allowed"));
        assert!(DOC.contains("escrow_transition_invalid"));
        assert!(DOC.contains("escrow_settlement_finalized"));
    }

    #[test]
    fn doc_contains_timeout_finality_race_matrix_contract() {
        assert!(DOC.contains("## Timeout/Finality Race Matrix Evidence"));
        assert!(DOC.contains("run_settlement_reconciliation_race_matrix.py"));
        assert!(DOC.contains("fixtures/escrow_reconciliation/finality_race_cases.json"));
    }

    #[test]
    fn regression_requires_missing_receipt_evidence_guard_marker() {
        // Regression: #678
        assert!(DOC.contains(
            "missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).",
        ));
    }

    #[test]
    fn regression_requires_ledger_reference_guard_marker() {
        // Regression: #717
        assert!(DOC.contains(
            "missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).",
        ));
    }

    #[test]
    fn regression_requires_transition_reason_code_guard_marker() {
        // Regression: #903
        assert!(DOC.contains(
            "transition reason-code drift and illegal transition acceptance fail closed (`Regression: #903`).",
        ));
    }
}

mod reputation_signal_routing_docs {
    const DOC: &str = include_str!("../../../docs/foundation/reputation-signal-routing.md");

    #[test]
    fn doc_contains_signal_integration_model() {
        assert!(DOC.contains("## Signal Integration Model"));
        assert!(DOC.contains("endorsements"));
        assert!(DOC.contains("disputes"));
        assert!(DOC.contains("verified capabilities"));
        assert!(DOC.contains("rank_agents_for_routing"));
        assert!(DOC.contains("rank_listings_by_reputation"));
    }

    #[test]
    fn doc_contains_error_handling_and_tiebreak_rules() {
        assert!(DOC.contains("## Validation and Error Handling"));
        assert!(DOC.contains("Invalid candidate DID"));
        assert!(DOC.contains("Missing reputation records"));
        assert!(DOC.contains("deterministic tie-break uses agent DID lexical order"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test reputation_signal_routing"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn doc_contains_dispute_evidence_contract() {
        assert!(DOC.contains("## Dispute Evidence Contract"));
        assert!(DOC.contains("generate_reputation_dispute_evidence_bundle.sh"));
        assert!(DOC.contains("check_reputation_dispute_policy.sh"));
        assert!(DOC.contains("reputation_dispute_contract.py"));
        assert!(DOC.contains("run_reputation_dispute_contract_lane.sh"));
        assert!(DOC.contains("run_reputation_dispute_deep_lane.sh"));
        assert!(DOC.contains("run_reputation_dispute_matrix.py"));
        assert!(DOC.contains("fixtures/reputation_dispute/replay_cases.json"));
    }

    #[test]
    fn doc_contains_signal_quarantine_evidence_contract() {
        assert!(DOC.contains("## Signal Quarantine Evidence Contract"));
        assert!(DOC.contains("generate_reputation_signal_quarantine_evidence_bundle.sh"));
        assert!(DOC.contains("check_reputation_signal_quarantine_policy.sh"));
        assert!(DOC.contains("run_reputation_signal_quarantine_contract_lane.sh"));
        assert!(DOC.contains("reputation_signal_quarantine_contract.py"));
    }

    #[test]
    fn regression_requires_did_tiebreak_rule() {
        // Regression: #211
        assert!(DOC.contains("Tie scores are resolved by DID lexical order."));
    }

    #[test]
    fn regression_requires_signal_quarantine_guard_marker() {
        // Regression: #935
        assert!(DOC.contains(
            "tampered reason keys/reason codes and ingestion-action mismatches force `NO-GO` (`Regression: #935`)."
        ));
    }

    #[test]
    fn regression_requires_reputation_dispute_evidence_guard_marker() {
        // Regression: #730
        assert!(DOC.contains(
            "tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`)."
        ));
    }
}
