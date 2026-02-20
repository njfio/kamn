// Consolidated docs-contract wave 3 harness.
// Migrated from 24 small per-doc suites to reduce file-surface overhead.

mod invariant_fuzz_strategy_docs {
    const DOC: &str = include_str!("../../../docs/testing/invariant-and-fuzz-strategy.md");

    #[test]
    fn invariant_strategy_docs_pin_transition_proptest_suite_sources() {
        assert!(DOC.contains("crates/kamn-core/tests/task_escrow_proptest_invariants.rs"));
        assert!(DOC.contains("crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs"));
    }

    #[test]
    fn invariant_strategy_docs_pin_transition_rejection_reason_codes() {
        assert!(DOC.contains("task_transition_invalid_edge"));
        assert!(DOC.contains("escrow_transition_invalid"));
        assert!(DOC.contains("runtime_peer_transition_invalid"));
    }
}

mod docs_contract_template_guidance_contract {
    const SUBTASK_TEMPLATE: &str = include_str!("../../../.github/ISSUE_TEMPLATE/subtask.md");

    #[test]
    fn conformance_subtask_template_contains_docs_contract_migration_checklist_markers() {
        assert!(SUBTASK_TEMPLATE.contains(
            "## Docs-Contract Matrix Migration Checklist (Required when docs-contract suites are touched)"
        ));
        assert!(SUBTASK_TEMPLATE
            .contains("docs_contract_matrix_migration_checklist_status=required-when-applicable"));
        assert!(SUBTASK_TEMPLATE
            .contains("docs_contract_matrix_case_inventory_status=declared-or-not-applicable"));
        assert!(SUBTASK_TEMPLATE.contains(
            "docs_contract_matrix_legacy_suite_retirement_status=verified-or-not-applicable"
        ));
    }
}

mod onchain_lifecycle_evidence_docs {
    const FOUNDATION_DOC: &str =
        include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");
    const DEVNET_DOC: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");
    const CI_DOC: &str = include_str!("../../../docs/ci/strategy.md");
    const README_DOC: &str = include_str!("../../../README.md");

    #[test]
    fn docs_include_onchain_lifecycle_evidence_bundle_contract_markers() {
        let docs = [FOUNDATION_DOC, DEVNET_DOC, CI_DOC, README_DOC];
        for doc in docs {
            assert!(doc.contains("run_onchain_lifecycle_evidence_bundle_lane.sh"));
            assert!(doc.contains("check_onchain_lifecycle_evidence_policy.py"));
            assert!(doc.contains("run_onchain_lifecycle_evidence_contract_lane.sh"));
            assert!(doc.contains("aggregate_bundle_lineage_mismatch"));
            assert!(doc.contains("finality_lineage_missing"));
            assert!(doc.contains("recovery_lineage_missing"));
            assert!(doc.contains("Regression: #3249"));
        }
    }
}

mod discord_bridge_approver_docs {
    const DOC: &str = include_str!("../../../docs/foundation/discord-bridge-approver-gating.md");

    #[test]
    fn doc_contains_discord_bridge_scope_and_quorum_contracts() {
        assert!(DOC.contains("# Discord Bridge Approver-Gated Outbound Flow"));
        assert!(DOC.contains("DiscordBridgeConfig"));
        assert!(DOC.contains("process_outbound_with_approvals(...)"));
        assert!(DOC.contains("approver quorum"));
    }

    #[test]
    fn doc_contains_bridge_replay_subset_validation_lane() {
        assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
        assert!(DOC.contains("--suites bridge_adapter,discord_bridge"));
        assert!(DOC.contains("bridge_replay_suites"));
    }

    #[test]
    fn regression_requires_signature_failure_fixture_reference() {
        // Regression: #587
        assert!(DOC.contains("signature-failure"));
        assert!(DOC.contains("Regression: #587"));
    }
}

mod openclaw_connector_docs {
    const DOC: &str =
        include_str!("../../../docs/foundation/openclaw-connector-reference-workflow.md");

    #[test]
    fn doc_contains_connector_contract_and_workflow_steps() {
        assert!(DOC.contains("## Connector Contract"));
        assert!(DOC.contains("registerOpenClawAgent(modelFamily)"));
        assert!(DOC.contains("runReferenceWorkflow(request)"));
        assert!(DOC.contains("1. send canonical message"));
        assert!(DOC.contains("3. create + release escrow"));
    }

    #[test]
    fn doc_contains_validation_rules_and_fast_lane_command() {
        assert!(DOC.contains("## Validation and Error Handling Rules"));
        assert!(DOC.contains("Workflow target must expose `openclaw` capability."));
        assert!(DOC.contains("npm --prefix packages/kamn-sdk test"));
    }

    #[test]
    fn regression_requires_empty_prompt_rejection_rule() {
        // Regression: #190
        assert!(DOC.contains("Empty prompt is rejected."));
    }
}

mod anti_spam_controls_docs {
    const DOC: &str = include_str!("../../../docs/foundation/anti-spam-controls.md");

    #[test]
    fn doc_contains_enforcement_rules_and_telemetry() {
        assert!(DOC.contains("## Enforcement Rules"));
        assert!(DOC.contains("Deposit gate"));
        assert!(DOC.contains("Per-agent rate limit"));
        assert!(DOC.contains("Suspension policy"));
        assert!(DOC.contains("## Telemetry Surface"));
        assert!(DOC.contains("rejected due to duplicate message ID"));
    }

    #[test]
    fn doc_contains_fast_validation_commands() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test anti_spam_controls"));
        assert!(DOC.contains("cargo clippy -- -D warnings"));
    }

    #[test]
    fn regression_requires_deposit_threshold_boundary_rule() {
        // Regression: #186
        assert!(DOC.contains("sender deposit must be at least `minimum_sybil_deposit`."));
    }
}

mod ci_caching_parallelism_docs {
    const DOC: &str = include_str!("../../../docs/foundation/ci-caching-parallelism.md");

    #[test]
    fn doc_contains_selector_scope_and_cache_guidance() {
        assert!(DOC.contains("shared-key: kamn-rust-ci-v1"));
        assert!(DOC.contains("run_ci_tool_checks"));
        assert!(DOC.contains("scripts/deploy/test_preflight_topology.sh"));
        assert!(DOC.contains("run_sdk_parity_matrix"));
    }

    #[test]
    fn regression_requires_performance_threshold_gate_commands() {
        // Regression: #595
        assert!(DOC.contains("generate_performance_smoke_report.sh --lane smoke|deep"));
        assert!(DOC.contains("check_performance_thresholds.sh --lane smoke|deep"));
        assert!(DOC.contains(".ci/performance-targets.env"));
    }

    #[test]
    fn regression_requires_sdk_parity_lane_guidance() {
        // Regression: #689
        assert!(DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
        assert!(DOC.contains("live_transport_parity_languages"));
        assert!(DOC.contains("run_live_transport_parity_rust_contract_tests"));
        assert!(DOC.contains("Regression: #689"));
    }
}

mod message_delivery_guards_docs {
    const DOC: &str = include_str!("../../../docs/foundation/message-delivery-guards.md");

    #[test]
    fn doc_contains_delivery_guard_scope_and_validation_rules() {
        assert!(DOC.contains("# Message Delivery Guards"));
        assert!(DOC.contains("MessageDeliveryGuards"));
        assert!(DOC.contains("Reject if `nonce` does not match sender expected nonce"));
    }

    #[test]
    fn doc_contains_durable_snapshot_store_contracts() {
        assert!(DOC.contains("## Durable Snapshot Stores"));
        assert!(DOC.contains("DurableGuardSnapshotBundle::capture"));
        assert!(DOC.contains("InMemoryDurableGuardSnapshotStore"));
        assert!(DOC.contains("FileDurableGuardSnapshotStore"));
    }

    #[test]
    fn regression_requires_corrupted_bundle_guard_marker() {
        // Regression: #679
        assert!(DOC.contains(
            "Truncated/corrupted durable bundle payloads fail closed (`Regression: #679`).",
        ));
    }
}

mod channel_permissions_retention_docs {
    const DOC: &str = include_str!("../../../docs/foundation/channel-permissions-retention.md");

    #[test]
    fn doc_contains_channel_permissions_scope_and_models() {
        assert!(DOC.contains("# Channel Permissions and Retention Policies"));
        assert!(DOC.contains("ChannelPermissionEngine"));
        assert!(DOC.contains("PermissionRule"));
    }

    #[test]
    fn regression_requires_allowlist_validation_rule() {
        // Regression: #458
        assert!(DOC.contains("Allowlist permission rules must not be empty"));
        assert!(DOC.contains("allowlist entries must be valid `kamn:did:agent:*` identifiers"));
        assert!(DOC.contains("malformed allowlist configuration is rejected (`Regression: #458`)"));
    }

    #[test]
    fn regression_requires_durable_bundle_corruption_guard_marker() {
        // Regression: #679
        assert!(DOC.contains(
            "Truncated/corrupted durable bundle payloads fail closed (`Regression: #679`).",
        ));
        assert!(DOC.contains("DurableGuardSnapshotBundle"));
    }
}

mod watchdog_node_docs {
    const DOC: &str = include_str!("../../../docs/foundation/watchdog-node-prototype.md");

    #[test]
    fn doc_contains_watchdog_scope_and_detection_rules() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("WatchdogNode"));
        assert!(DOC.contains("## Detection Rules"));
        assert!(DOC.contains("Invalid block parent"));
        assert!(DOC.contains("Quorum anomaly"));
        assert!(DOC.contains("Censorship signal"));
    }

    #[test]
    fn doc_contains_snapshot_and_validation_semantics() {
        assert!(DOC.contains("## Snapshot Semantics"));
        assert!(DOC.contains("WatchdogSnapshot"));
        assert!(DOC.contains("## Validation and Error Handling"));
        assert!(DOC.contains("Config rejects zero quorum threshold."));
    }

    #[test]
    fn regression_requires_single_recipient_censorship_exclusion_rule() {
        // Regression: #204
        assert!(DOC
            .contains("single-recipient deliveries are excluded from censorship classification."));
    }
}

mod agent_interop_wave_docs {
    const DOC: &str = include_str!("../../../docs/planning/agent-interop-wave.md");

    #[test]
    fn doc_contains_did_lifecycle_contract_lane_commands() {
        assert!(DOC.contains("## DID Lifecycle Mutation Contract Lane (Issue #889)"));
        assert!(DOC.contains("did_lifecycle_mutation_transactions"));
        assert!(DOC.contains("run_did_registry_contract_lane.sh"));
        assert!(DOC.contains("did_lifecycle_mutation_reason_codes:GO:v1"));
    }

    #[test]
    fn regression_requires_did_lifecycle_drift_fail_closed_marker() {
        // Regression: #889
        assert!(DOC.contains("Regression: #889"));
    }

    #[test]
    fn doc_contains_lifecycle_operator_binding_contract_lane_commands() {
        assert!(DOC.contains("## Lifecycle Operator-Binding Policy Contract Lane (Issue #890)"));
        assert!(DOC.contains("run_lifecycle_operator_binding_contract_lane.sh"));
        assert!(DOC.contains("did_lifecycle_operator_binding_reason_codes:GO:v1"));
    }

    #[test]
    fn regression_requires_lifecycle_operator_binding_drift_fail_closed_marker() {
        // Regression: #890
        assert!(DOC.contains("Regression: #890"));
    }
}

mod token_config_docs {
    const DOC: &str = include_str!("../../../docs/foundation/token-config.md");

    #[test]
    fn doc_contains_token_launch_handoff_evidence_contract() {
        assert!(DOC.contains("## Token Launch Handoff Evidence Contract"));
        assert!(DOC.contains("generate_token_launch_handoff_evidence_bundle.sh"));
        assert!(DOC.contains("check_token_launch_handoff_policy.sh"));
        assert!(DOC.contains("token_launch_handoff_contract_lane_contract.py"));
        assert!(DOC.contains("run_token_launch_handoff_contract_lane.sh"));
        assert!(DOC.contains("run_token_launch_handoff_deep_lane.sh"));
        assert!(DOC.contains("fixtures/token_launch/handoff_invariant_cases.json"));
    }

    #[test]
    fn regression_requires_token_launch_handoff_guard_marker() {
        // Regression: #714
        assert!(DOC.contains(
            "supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`)."
        ));
    }

    #[test]
    fn regression_requires_token_launch_shared_contract_marker() {
        // Regression: #1270
        assert!(DOC.contains(
            "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1270`)."
        ));
    }
}

mod runtime_module_extraction_roadmap_docs {
    const ROADMAP: &str =
        include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

    #[test]
    fn roadmap_tracks_runtime_decomposition_tranche4_snapshot_module_extraction() {
        assert!(ROADMAP.contains("Task #3090"));
        assert!(ROADMAP.contains("Task #3092"));
        assert!(ROADMAP.contains("Subtask #3093"));
        assert!(ROADMAP.contains("crates/kamn-core/src/runtime_snapshot_store.rs"));
        assert!(ROADMAP.contains("runtime_module_extraction_contract.rs"));
        assert!(ROADMAP.contains("runtime_network_docs.rs"));
    }

    #[test]
    fn roadmap_tracks_runtime_decomposition_tranche5_recovery_guard_module_extraction() {
        assert!(ROADMAP.contains("Task #3129"));
        assert!(ROADMAP.contains("Subtask #3130"));
        assert!(ROADMAP.contains("crates/kamn-core/src/runtime_recovery_guard.rs"));
        assert!(ROADMAP.contains("runtime_module_extraction_contract.rs"));
        assert!(ROADMAP.contains("runtime_network_docs.rs"));
    }

    #[test]
    fn roadmap_tracks_runtime_decomposition_tranche6_peer_coordination_module_extraction() {
        assert!(ROADMAP.contains("Task #3145"));
        assert!(ROADMAP.contains("Subtask #3155"));
        assert!(ROADMAP.contains("crates/kamn-core/src/runtime_peer_coordination.rs"));
        assert!(ROADMAP.contains("runtime_module_extraction_contract.rs"));
        assert!(ROADMAP.contains("runtime_network_docs.rs"));
    }
}

mod upgrade_orchestration_docs {
    const DOC: &str =
        include_str!("../../../docs/foundation/version-upgrade-orchestration-audit.md");

    #[test]
    fn doc_contains_upgrade_orchestrator_scope_and_models() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("VersionUpgradeOrchestrator"));
        assert!(DOC.contains("UpgradeAuditEventKind"));
        assert!(DOC.contains("UpgradeOrchestrationError"));
    }

    #[test]
    fn doc_contains_upgrade_gating_and_audit_rules() {
        assert!(DOC.contains("## Upgrade Gating Rules"));
        assert!(DOC.contains("## Governance Audit View Rules"));
        assert!(DOC.contains("Activation requires:"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test upgrade_orchestration"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_activation_quorum_gating_rule() {
        // Regression: #193
        assert!(DOC.contains("sufficient unique validator approvals"));
    }
}

mod content_retention_tombstones_docs {
    const DOC: &str = include_str!("../../../docs/foundation/content-retention-tombstones.md");

    #[test]
    fn doc_contains_retention_class_and_lifecycle_scope() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("ContentRetentionClass"));
        assert!(DOC.contains("ContentLifecycleManager"));
        assert!(DOC.contains("ContentCleanupActionKind"));
    }

    #[test]
    fn doc_contains_cleanup_and_deleted_reference_rules() {
        assert!(DOC.contains("## Lifecycle and Cleanup Rules"));
        assert!(DOC.contains("Active` -> `Expired` -> `Tombstoned` -> `Purged"));
        assert!(DOC.contains("## Deleted Reference Semantics"));
        assert!(DOC.contains("assert_uri_accessible(...)"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test content_retention_tombstones"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_deleted_reference_replay_block_rule() {
        // Regression: #163
        assert!(DOC.contains("Deleted/tombstoned references remain blocked under replay attempts."));
    }
}

mod operator_permissioned_actions_docs {
    const DOC: &str = include_str!("../../../docs/foundation/operator-permissioned-actions.md");

    #[test]
    fn doc_contains_scope_and_service_contracts() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("PermissionedOperatorActionService"));
        assert!(DOC.contains("OperatorActionAuditRecord"));
        assert!(DOC.contains("OperatorActionServiceError"));
    }

    #[test]
    fn doc_contains_binding_authorization_rules() {
        assert!(DOC.contains("## Authorization Rules"));
        assert!(DOC.contains("OperatorBindingAction::Configure"));
        assert!(DOC.contains("OperatorBindingAction::ReadHistory"));
        assert!(DOC.contains("Unauthorized requests return explicit binding errors"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test operator_permissioned_actions"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_post_revoke_blocking_rule() {
        // Regression: #199
        assert!(DOC.contains("Revoked bindings cannot be reused"));
    }
}

mod content_replication_repair_docs {
    const DOC: &str = include_str!("../../../docs/foundation/content-replication-repair.md");

    #[test]
    fn doc_contains_policy_and_repair_scope() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("ContentReplicationPolicy"));
        assert!(DOC.contains("ContentReplicationManager"));
        assert!(DOC.contains("ContentRepairAction"));
    }

    #[test]
    fn doc_contains_health_and_retry_rules() {
        assert!(DOC.contains("## Availability and Repair Rules"));
        assert!(DOC.contains("Healthy"));
        assert!(DOC.contains("Degraded"));
        assert!(DOC.contains("Unavailable"));
        assert!(DOC.contains("suppresses duplicate repair actions while a repair is pending"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test content_replication_repair"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_duplicate_repair_suppression_rule() {
        // Regression: #167
        assert!(DOC.contains("suppresses duplicate repair actions while a repair is pending"));
    }
}

mod content_retrieval_access_cache_docs {
    const DOC: &str = include_str!("../../../docs/foundation/content-retrieval-access-cache.md");

    #[test]
    fn doc_contains_retrieval_scope_and_engine_contract() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("ContentRetrievalConfig"));
        assert!(DOC.contains("ContentRetrievalRequest"));
        assert!(DOC.contains("ContentRetrievalEngine"));
    }

    #[test]
    fn doc_contains_authorization_and_cache_binding_rules() {
        assert!(DOC.contains("## Authorization and Cache Rules"));
        assert!(DOC.contains("grant_task_read"));
        assert!(DOC.contains("ChannelPermissionEngine"));
        assert!(DOC.contains("cache key binds `requester + scope + cid`."));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test content_retrieval_access_cache"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_scope_bound_cache_rule() {
        // Regression: #165
        assert!(DOC.contains(
            "Cache entries cannot be reused across different requester/scope combinations."
        ));
    }
}

mod content_storage_adapter_docs {
    const DOC: &str = include_str!("../../../docs/foundation/content-storage-adapter.md");

    #[test]
    fn doc_contains_adapter_contract_scope_and_helpers() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("ContentStorageAdapter"));
        assert!(DOC.contains("InMemoryContentAdapter"));
        assert!(DOC.contains("content_uri_for_cid"));
        assert!(DOC.contains("cid_from_content_uri"));
    }

    #[test]
    fn doc_contains_integrity_and_task_artifact_integration_rules() {
        assert!(DOC.contains("## Integrity Verification Rules"));
        assert!(DOC.contains("IntegrityMismatch"));
        assert!(DOC.contains("## Task Artifact Integration Path"));
        assert!(DOC.contains("TaskArtifactRegistry::integrity_fingerprint"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --test content_storage_adapter"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_tamper_detection_behavior() {
        // Regression: #169
        assert!(
            DOC.contains("Corruption/tampering returns `ContentStorageError::IntegrityMismatch`.")
        );
    }
}

mod task_escrow_suite_modularization_contract {
    use std::path::Path;

    const ROOT_SUITE: &str = include_str!("task_escrow_proptest_invariants.rs");

    #[test]
    fn root_harness_declares_task_and_escrow_domain_modules() {
        assert!(ROOT_SUITE
            .contains("#[path = \"task_escrow_proptest_invariants/shared.rs\"]\nmod shared;"));
        assert!(ROOT_SUITE.contains(
            "#[path = \"task_escrow_proptest_invariants/task_domain.rs\"]\nmod task_domain;"
        ));
        assert!(ROOT_SUITE.contains(
            "#[path = \"task_escrow_proptest_invariants/escrow_domain.rs\"]\nmod escrow_domain;"
        ));
    }

    #[test]
    fn modularized_suite_files_exist_and_are_tracked() {
        assert!(Path::new("tests/task_escrow_proptest_invariants/shared.rs").is_file());
        assert!(Path::new("tests/task_escrow_proptest_invariants/task_domain.rs").is_file());
        assert!(Path::new("tests/task_escrow_proptest_invariants/escrow_domain.rs").is_file());
    }

    #[test]
    fn testing_strategy_doc_records_suite_modularization_conventions() {
        let doc = std::fs::read_to_string("../../docs/testing/strategy.md")
            .expect("testing strategy doc must exist for suite modularization policy");
        assert!(doc.contains("task_escrow_proptest_invariants"));
        assert!(doc.contains("domain modules"));
    }
}

mod kolme_runtime_architecture_docs {
    const DOC: &str = include_str!("../../../docs/foundation/kolme-runtime-architecture.md");
    const README: &str = include_str!("../../../README.md");

    #[test]
    fn architecture_doc_contains_runtime_flow_and_signer_boundaries() {
        assert!(DOC.contains("## Runtime Flow Diagram"));
        assert!(DOC.contains("```mermaid"));
        assert!(DOC.contains("graph TD"));
        assert!(DOC.contains("kamn-node"));
        assert!(DOC.contains("kamn-core"));
        assert!(DOC.contains("kamn-kolme"));
        assert!(DOC.contains("KolmeRuntimeCommitLiveProvider"));
        assert!(DOC.contains("managed-external signer backend"));
        assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX"));
        assert!(DOC.contains("KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY"));
    }

    #[test]
    fn architecture_doc_contains_module_ownership_map() {
        assert!(DOC.contains("## Ownership Map"));
        assert!(DOC.contains("crates/kamn-node/src/runtime_kolme_live.rs"));
        assert!(DOC.contains("crates/kamn-node/src/signer.rs"));
        assert!(DOC.contains("crates/kamn-core/src/kolme_runtime_commit.rs"));
        assert!(DOC.contains("crates/kamn-kolme/src/live_provider_pipeline.rs"));
    }

    #[test]
    fn readme_references_architecture_doc() {
        assert!(README.contains("docs/foundation/kolme-runtime-architecture.md"));
        assert!(
            README.contains("docs/foundation/kolme-runtime-architecture.md#runtime-flow-diagram")
        );
        assert!(README.contains("docs/foundation/kolme-runtime-architecture.md#ownership-map"));
    }
}

mod failover_runbook_docs {
    const RUNBOOK: &str = include_str!("../../../docs/foundation/multi-az-failover-runbook.md");

    #[test]
    fn runbook_contains_topology_section() {
        assert!(RUNBOOK.contains("## Multi-AZ Topology"));
        assert!(RUNBOOK.contains("AZ-a"));
        assert!(RUNBOOK.contains("AZ-b"));
        assert!(RUNBOOK.contains("AZ-c"));
    }

    #[test]
    fn runbook_contains_failover_steps() {
        assert!(RUNBOOK.contains("## Processor Failover Procedure"));
        assert!(RUNBOOK.contains("1. Detect processor failure"));
        assert!(RUNBOOK.contains("2. Validate listener and approver quorum"));
        assert!(RUNBOOK.contains("3. Promote standby processor"));
        assert!(RUNBOOK.contains("4. Verify chain continuity"));
    }

    #[test]
    fn runbook_contains_verification_checklist() {
        assert!(RUNBOOK.contains("## Verification Checklist"));
        assert!(RUNBOOK.contains("State hash continuity confirmed"));
        assert!(RUNBOOK.contains("No duplicate block production"));
    }

    #[test]
    fn regression_requires_topology_bundle_command_surface() {
        // Regression: #579
        assert!(RUNBOOK.contains("## Topology Bundle Command Surface"));
        assert!(RUNBOOK.contains("scripts/deploy/generate_bundle.sh"));
        assert!(RUNBOOK.contains("scripts/deploy/preflight_topology.sh --bundle-file"));
    }
}

mod secure_coding_docs {
    const DOC: &str = include_str!("../../../docs/security/secure-coding.md");

    #[test]
    fn doc_contains_panic_path_reachability_and_unsafe_fallback_markers() {
        assert!(DOC.contains("# Secure Coding"));
        assert!(DOC.contains("panic_path_reachability_policy=fail_closed"));
        assert!(DOC.contains("unsafe_fallback_default_policy=fail_closed"));
        assert!(DOC.contains(
            "scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --output-json /tmp/no-production-expect-report.json"
        ));
        assert!(DOC.contains(
            "production_panic_path_violation_markers=.expect(,panic!,unreachable!,unsafe_env_fallback_default"
        ));
        assert!(DOC
            .contains("production_panic_path_violation_class=panic_reachability|unsafe_fallback"));
        assert!(DOC.contains(
            "panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1"
        ));
        assert!(DOC.contains(
            "panic_replacement_reason_codes_csv=scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default"
        ));
        assert!(DOC.contains("panic_replacement_reason_codes_value=none|<csv>"));
        assert!(DOC.contains(
            "panic_replacement_reason_class=stable|panic_reachability|unsafe_fallback|mixed|configuration"
        ));
        assert!(DOC.contains("runtime_panic_replacement_evidence_status=verified|violation"));
        assert!(DOC.contains("runtime_panic_replacement_evidence_violation_count=<n>"));
        assert!(DOC.contains("runtime_panic_replacement_evidence_files_csv=none|<csv>"));
        assert!(DOC.contains(
            "runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv"
        ));
    }
}

mod channel_models_and_permissions_docs {
    const DOC: &str = include_str!("../../../docs/foundation/channel-models-and-permissions.md");

    #[test]
    fn doc_contains_channel_models_and_permissions_scope() {
        assert!(DOC.contains("# Channel Models and Permissions Contract Rules"));
        assert!(DOC.contains("run_channel_policy_contract_lane.sh"));
        assert!(DOC.contains("channel_policy_contract_lane_contract.py"));
        assert!(DOC.contains("channel_lifecycle_contract_lane_contract.py"));
        assert!(DOC.contains("run_channel_retention_redaction_contract_lane.sh"));
        assert!(DOC.contains("channel_permissions_retention"));
    }

    #[test]
    fn regression_requires_channel_policy_bypass_marker() {
        // Regression: #929
        assert!(DOC.contains("unauthorized channel policy bypass is rejected (`Regression: #929`)"));
        assert!(DOC.contains("test_run_channel_policy_contract_lane.sh"));
    }

    #[test]
    fn regression_requires_channel_policy_shared_contract_marker() {
        // Regression: #1274
        assert!(DOC.contains(
            "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1274`)"
        ));
    }

    #[test]
    fn regression_requires_channel_lifecycle_shared_contract_marker() {
        // Regression: #1290
        assert!(DOC.contains(
            "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1290`)"
        ));
    }
}

mod kolme_fork_api_contract_inventory_docs {
    const DOC: &str = include_str!("../../../docs/research/kolme-fork-api-contract-inventory.md");

    #[test]
    fn inventory_captures_kolme_fork_base_api_routes_and_kamn_expectations() {
        assert!(DOC.contains("`/broadcast`"));
        assert!(DOC.contains("`/get-next-nonce`"));
        assert!(DOC.contains("`/block/{height}`"));
        assert!(DOC.contains("`/notifications`"));
        assert!(DOC.contains("`/fork-info`"));
        assert!(DOC.contains("`/healthz`"));
        assert!(DOC.contains("`/broadcast/runtime-commit`"));
        assert!(DOC.contains("`/runtime-commit/status`"));
    }

    #[test]
    fn inventory_lists_integration_gaps_and_follow_up_issue_links() {
        assert!(DOC.contains("Gap: runtime_commit_submit_endpoint_mismatch"));
        assert!(DOC.contains("Gap: runtime_commit_payload_shape_mismatch"));
        assert!(DOC.contains("Gap: runtime_commit_finality_endpoint_missing"));
        assert!(DOC.contains("Gap: block_fallback_schema_mismatch"));
        assert!(DOC.contains("Follow-up Tasks"));
        assert!(DOC.contains("- #1502"));
        assert!(DOC.contains("- #1503"));
        assert!(DOC.contains("- #1504"));
    }

    #[test]
    fn regression_marker_documents_fork_contract_inventory_baseline() {
        // Regression: #1501
        assert!(
            DOC.contains(
                "KAMN-to-kolme_fork endpoint/method/payload contract inventory remains synchronized with code-level integration assumptions (`Regression: #1501`)."
            )
        );
    }
}

mod kolme_live_integration_docs {
    const DOC: &str = include_str!("../../../docs/architecture/kolme-live-integration.md");

    #[test]
    fn doc_contains_process_isolation_marker_contracts() {
        assert!(DOC.contains("transport_convergence_status"));
        assert!(DOC.contains("libp2p_process_isolation_status"));
        assert!(DOC.contains("libp2p_two_node_process_isolated_status"));
        assert!(DOC.contains("libp2p_three_node_process_isolated_status"));
        assert!(DOC.contains("local_heavy_runtime_budget_status"));
        assert!(DOC.contains("elapsed_seconds"));
        assert!(DOC.contains("max_seconds"));
        assert!(DOC.contains("command_max_seconds"));
        assert!(DOC.contains("runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider"));
    }

    #[test]
    fn doc_contains_process_isolation_fail_closed_reasons() {
        assert!(
            DOC.contains("local_full_stack_integration_policy_reason_taxonomy_version_mismatch")
        );
        assert!(DOC.contains(
            "local_full_stack_integration_policy_libp2p_process_isolation_status_mismatch"
        ));
        assert!(DOC.contains(
            "local_full_stack_integration_policy_libp2p_two_node_process_isolated_status_mismatch"
        ));
        assert!(DOC.contains(
            "local_full_stack_integration_policy_libp2p_three_node_process_isolated_status_mismatch"
        ));
        assert!(DOC.contains(
            "local_full_stack_integration_policy_libp2p_summary_three_node_partition_rejoin_status_mismatch"
        ));
        assert!(DOC.contains(
            "local_full_stack_integration_policy_libp2p_summary_three_node_publish_drop_status_mismatch"
        ));
        assert!(DOC.contains("local_full_stack_integration_policy_runtime_budget_status_mismatch"));
        assert!(DOC.contains("local_full_stack_integration_policy_runtime_budget_exceeded"));
    }
}

mod task_state_machine_docs {
    const DOC: &str = include_str!("../../../docs/foundation/task-state-machine.md");

    #[test]
    fn doc_contains_task_lifecycle_scope_and_transition_map() {
        assert!(DOC.contains("# Task State Machine and Transition Validator"));
        assert!(DOC.contains("TaskLifecycle::new(task_id)"));
        assert!(DOC.contains("## Supported Transitions"));
    }

    #[test]
    fn doc_contains_transition_evidence_reason_code_contract() {
        assert!(DOC.contains("## Transition Evidence and Reason-Code Contract"));
        assert!(DOC.contains("transition_with_evidence(TaskTransition)"));
        assert!(DOC.contains("TaskTransitionEvidence"));
        assert!(DOC.contains("task_transition_allowed"));
        assert!(DOC.contains("task_transition_invalid_edge"));
        assert!(DOC.contains("task_transition_terminal_state"));
        assert!(DOC.contains("task_history_invalid"));
        assert!(DOC.contains("task_id_empty"));
    }

    #[test]
    fn doc_includes_transition_contract_validation_commands() {
        assert!(DOC.contains("cargo test -p kamn-core --test task_state_machine"));
        assert!(DOC.contains("cargo test -p kamn-core --test task_escrow_transition_contracts"));
        assert!(DOC.contains("cargo test -p kamn-core --test task_state_machine_docs"));
    }

    #[test]
    fn regression_marker_for_transition_reason_code_drift_is_present() {
        // Regression: #903
        assert!(DOC.contains(
            "transition reason-code drift and illegal transition acceptance fail closed (`Regression: #903`)."
        ));
    }
}

mod telegram_bridge_listener_docs {
    const DOC: &str =
        include_str!("../../../docs/foundation/telegram-bridge-listener-validation.md");

    #[test]
    fn doc_contains_telegram_bridge_scope_and_listener_contracts() {
        assert!(DOC.contains("# Telegram Bridge Listener-Validated Inbound Flow"));
        assert!(DOC.contains("TelegramBridgeConfig"));
        assert!(DOC.contains("process_inbound_to_envelope(...)"));
        assert!(DOC.contains("listener DID must be authorized"));
        assert!(DOC.contains("webhook token must match configured Telegram auth token"));
        assert!(DOC.contains("checkpoint must be monotonic per `external_channel_id`"));
    }

    #[test]
    fn doc_contains_bridge_replay_subset_validation_lane() {
        assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
        assert!(DOC.contains("--suites bridge_adapter,telegram_bridge"));
        assert!(DOC.contains("bridge_replay_suites"));
        assert!(DOC.contains("run_telegram_ingress_contract_lane.sh"));
        assert!(DOC.contains("run_telegram_ingress_deep_lane.sh"));
    }

    #[test]
    fn regression_requires_replay_fixture_reference() {
        // Regression: #587
        assert!(DOC.contains("duplicate replay"));
        assert!(DOC.contains("Regression: #587"));
    }

    #[test]
    fn regression_requires_forged_webhook_and_checkpoint_rejection_rule() {
        // Regression: #621
        assert!(DOC.contains(
            "forged webhook tokens and replayed/out-of-order checkpoints are rejected (`Regression: #621`)."
        ));
    }
}

mod rehearsal_rollback_governance_docs {
    const PLAN: &str =
        include_str!("../../../docs/plans/2026-02-14-production-service-next-steps.md");
    const CI_STRATEGY: &str = include_str!("../../../docs/ci/strategy.md");
    const INCIDENT_READINESS: &str = include_str!("../../../docs/ops/incident-readiness.md");

    #[test]
    fn plan_contains_r27_19_rehearsal_rollback_ci_smoke_closure_markers() {
        assert!(PLAN.contains("### R27.19 Rehearsal/Rollback CI Smoke Governance Closure"));
        assert!(PLAN.contains("Active chain: `#4145 -> #4149 -> (#4156, #4157)`."));
        assert!(PLAN.contains("rehearsal_promotion_ci_smoke_convergence_status=verified"));
        assert!(PLAN.contains(
            "rehearsal_promotion_ci_smoke_reason_taxonomy_version=kamn.ci.rehearsal-promotion-ci-smoke-convergence-reason-taxonomy.v1"
        ));
        assert!(PLAN.contains("rehearsal_promotion_ci_smoke_max_seconds=120"));
        assert!(PLAN.contains("rehearsal_promotion_local_heavy_max_seconds=900"));
        assert!(PLAN.contains(
            "python3 scripts/deploy/check_upgrade_rehearsal_lineage_policy.py --bundle-file /tmp/gonogo-milestone.json --expected-final-decision GO"
        ));
        assert!(PLAN.contains("bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh"));
    }

    #[test]
    fn ci_and_ops_docs_keep_rehearsal_boundary_markers_in_sync() {
        let required_markers = [
            "rehearsal_boundary_reason_codes_csv=rehearsal_boundary_ci_smoke_seconds_exceeded,rehearsal_boundary_local_heavy_opt_in_missing,rehearsal_runbook_contract_parity_mismatch",
            "rehearsal_boundary_ci_smoke_max_seconds=120",
            "rehearsal_boundary_local_heavy_max_seconds=900",
        ];

        for marker in required_markers {
            assert!(
                CI_STRATEGY.contains(marker),
                "docs/ci/strategy.md missing marker: {marker}"
            );
            assert!(
                INCIDENT_READINESS.contains(marker),
                "docs/ops/incident-readiness.md missing marker: {marker}"
            );
        }
    }
}

mod message_proof_anchoring_docs {
    const DOC: &str = include_str!("../../../docs/foundation/message-proof-anchoring.md");

    #[test]
    fn doc_contains_anchor_service_contracts() {
        assert!(DOC.contains("MessageProofAnchoringService"));
        assert!(DOC.contains("anchor_message_proof_via_chain_adapter"));
        assert!(DOC.contains("idempotency_key_for_anchor"));
        assert!(DOC.contains("NewSubmission"));
        assert!(DOC.contains("RetryableInFlight"));
        assert!(DOC.contains("FinalizedNoRetry"));
        assert!(DOC.contains("ConflictNoRetry"));
    }

    #[test]
    fn doc_contains_kolme_and_outcome_contracts() {
        assert!(DOC.contains("KolmeMessageProofChainAdapter"));
        assert!(DOC.contains("InMemoryMessageProofChainAdapter"));
        assert!(DOC.contains("Submitted(receipt)"));
        assert!(DOC.contains("Duplicate(receipt)"));
        assert!(DOC.contains("Rejected { reason }"));
        assert!(DOC.contains("FinalizedNoOp"));
    }

    #[test]
    fn regression_doc_marks_conflicting_idempotency_fail_closed_guard() {
        // Regression: #2941
        assert!(DOC.contains("Regression: #2941"));
    }

    #[test]
    fn regression_doc_marks_mismatch_tamper_reason_taxonomy_and_ci_boundary_contracts() {
        // Regression: #4419
        assert!(DOC.contains("Regression: #4419"));
        assert!(DOC.contains(
            "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1"
        ));
        assert!(DOC.contains(
            "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required"
        ));
        assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified"));
    }
}

mod channel_models_docs {
    const DOC: &str = include_str!("../../../docs/foundation/channel-models.md");

    #[test]
    fn doc_contains_core_channel_models_and_operations() {
        assert!(DOC.contains("## Core Models"));
        assert!(DOC.contains("ChannelType"));
        assert!(DOC.contains("ChannelMetadata"));
        assert!(DOC.contains("ChannelStore"));
        assert!(DOC.contains("## Supported Operations"));
        assert!(DOC.contains("create_group(channel_id, creator, members, admins)"));
    }

    #[test]
    fn doc_contains_snapshot_persistence_and_restore_contract_rules() {
        assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
        assert!(DOC.contains("export_snapshot()"));
        assert!(DOC.contains("restore_snapshot(snapshot)"));
        assert!(DOC.contains("ChannelSnapshotStore"));
        assert!(DOC.contains("recover_latest_and_repair()"));
        assert!(DOC.contains("CHANNEL_SNAPSHOT_SCHEMA_VERSION"));
    }

    #[test]
    fn doc_contains_fast_and_cost_effective_validation_lane_commands() {
        assert!(DOC.contains("## Fast and Cost-Effective Validation"));
        assert!(DOC.contains("cargo test -p kamn-core --lib channel_models::tests::"));
        assert!(DOC.contains("cargo test -p kamn-core --test channel_models"));
        assert!(DOC.contains("cargo test -p kamn-core --test channel_models_docs"));
        assert!(DOC.contains("bash scripts/channel/run_channel_lifecycle_contract_lane.sh"));
        assert!(DOC.contains(
            "cargo test -p kamn-core --lib channel_models::tests::performance_channel_snapshot_deep_lane_stress -- --ignored"
        ));
        assert!(DOC.contains("bash scripts/channel/run_channel_lifecycle_deep_lane.sh"));
        assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
    }

    #[test]
    fn regression_requires_channel_snapshot_restore_guard_rules() {
        // Regression: #617
        assert!(DOC.contains("duplicate channel IDs on restore are rejected (`Regression: #617`)"));
        assert!(DOC.contains("admin/member mismatch on restore is rejected (`Regression: #617`)"));
    }
}

mod invariants_docs {
    const DOC: &str = include_str!("../../../docs/foundation/invariants.md");

    #[test]
    fn doc_contains_runtime_invariant_harness_coverage_contract() {
        assert!(DOC.contains("## Runtime Invariant Harness Coverage (Issue #897)"));
        assert!(DOC.contains("run_lifecycle_property_contract_lane.sh"));
        assert!(DOC.contains("kamn.runtime.lifecycle-property-contract-report.v1"));
        assert!(DOC.contains("kamn.runtime.lifecycle-property-replay-metadata.v1"));
        assert!(DOC.contains("lifecycle_property_replay:v1"));
        assert!(DOC.contains("run_input_mutation_contract_lane.sh"));
        assert!(DOC.contains("kamn.runtime.input-mutation-contract-report.v1"));
        assert!(DOC.contains("input_mutation_replay:v1"));
        assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
        assert!(DOC.contains("kamn.runtime.concurrency-mutation-contract-report.v1"));
        assert!(DOC.contains("concurrency_mutation_replay:v1"));
        assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
        assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
        assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
    }

    #[test]
    fn regression_requires_lifecycle_property_replay_metadata_contract_markers() {
        // Regression: #1605
        assert!(DOC.contains("kamn.runtime.lifecycle-property-replay-metadata.v1"));
        assert!(DOC.contains("generated_sequence_bounds"));
        assert!(DOC.contains("executed_cases"));
    }

    #[test]
    fn regression_requires_dispute_refund_property_and_concurrency_contract_markers() {
        // Regression: #904
        assert!(DOC.contains("## Dispute/Refund Property and Concurrency Contracts (Issue #904)"));
        assert!(DOC.contains("dispute_refund_transition_contracts"));
        assert!(DOC.contains("run_lifecycle_property_contract_lane.sh"));
        assert!(DOC.contains("run_concurrency_state_mutation_contract_lane.sh"));
        assert!(DOC.contains("Regression: #904"));
    }

    #[test]
    fn regression_requires_zk_witness_mutation_contract_markers() {
        // Regression: #994
        assert!(DOC.contains("run_zk_witness_mutation_contract_lane.sh"));
        assert!(DOC.contains("run_zk_witness_mutation_deep_lane.sh"));
        assert!(DOC.contains("KAMN_RUNTIME_ZK_WITNESS_MUTATION_DEEP"));
    }
}

mod versioning_compatibility_docs {
    const POLICY: &str =
        include_str!("../../../docs/foundation/versioning-compatibility-matrix.md");

    #[test]
    fn policy_defines_semantic_versions_for_chain_app_and_sdks() {
        assert!(POLICY.contains("## Semantic Versioning Policy"));
        assert!(POLICY.contains("Chain protocol version follows MAJOR.MINOR.PATCH."));
        assert!(POLICY.contains("App-state schema version follows MAJOR.MINOR.PATCH."));
        assert!(
            POLICY.contains("SDK versions (Rust, Python, TypeScript) follow MAJOR.MINOR.PATCH.")
        );
    }

    #[test]
    fn policy_contains_compatibility_matrix_with_upgrade_and_downgrade_expectations() {
        assert!(POLICY.contains("## Compatibility Matrix"));
        assert!(POLICY.contains("| Chain Protocol | App-State Schema | Node Binary | SDK Family | Upgrade Expectation | Downgrade Expectation |"));
        assert!(POLICY.contains("Same major version upgrade: supported with migration plan."));
        assert!(POLICY
            .contains("Cross-major upgrade: requires governance approval and staged rollout."));
        assert!(POLICY.contains("Downgrade across major versions: blocked."));
    }

    #[test]
    fn policy_defines_support_and_deprecation_windows() {
        assert!(POLICY.contains("## Support and Deprecation Windows"));
        assert!(POLICY.contains("Current minor (N) and previous minor (N-1) are supported."));
        assert!(
            POLICY.contains("Anything older than N-1 is deprecated and no-go for new rollouts.")
        );
    }

    #[test]
    fn policy_defines_governance_parameter_compatibility_contract() {
        assert!(POLICY.contains("## Governance Parameter Compatibility Policy"));
        assert!(POLICY.contains("| Parameter Key | Allowed Range | Minimum Supported Version |"));
        assert!(POLICY.contains("| `listener.quorum` | `[1, 7]` | `1.0.0` |"));
        assert!(POLICY.contains("| `watchdog.delivery_ratio_bps` | `[9000, 9999]` | `1.1.0` |"));
    }

    #[test]
    fn regression_requires_incompatible_downgrade_no_go_rule() {
        // Regression: #175
        assert!(POLICY.contains("Incompatible downgrade decision: NO-GO."));
        assert!(POLICY.contains(
            "Referenced by governance workflow: docs/foundation/release-gonogo-checklist.md"
        ));
        assert!(POLICY.contains(
            "Referenced by migration/rollback workflow: docs/foundation/upgrade-rollback-runbook.md"
        ));
    }
}

mod cross_chain_bridge_adapters_docs {
    const DOC: &str = include_str!("../../../docs/foundation/cross-chain-bridge-adapters.md");

    #[test]
    fn doc_covers_adapter_and_receipt_normalization_scopes() {
        assert!(DOC.contains("# Cross-Chain Bridge Adapters"));
        assert!(DOC.contains("CrossChainBridgeEngine"));
        assert!(DOC.contains("normalize_cross_chain_receipt(...)"));
    }

    #[test]
    fn doc_lists_ethereum_and_near_finality_rules() {
        assert!(DOC.contains("## Receipt Finality Normalization Rules (Ethereum / Near)"));
        assert!(DOC.contains("finalized` or `safe` with at least `12` confirmations"));
        assert!(DOC.contains("`final` -> `Final`"));
    }

    #[test]
    fn doc_contains_outbound_intent_attestation_and_idempotency_rules() {
        assert!(DOC.contains("## Outbound Intent Attestation and Retry Idempotency Rules"));
        assert!(DOC.contains("idempotency key must be `idemp:<value>`"));
        assert!(DOC.contains("duplicate request flag forces deterministic `NO-GO`"));
        assert!(DOC.contains("Regression: #742"));
    }

    #[test]
    fn doc_includes_cross_chain_receipt_finality_test_command() {
        assert!(DOC.contains("cargo test -p kamn-core --test cross_chain_receipt_finality"));
        assert!(DOC.contains("cross_chain_outbound_intent_contract.py"));
        assert!(DOC.contains("test_generate_cross_chain_outbound_intent_evidence_bundle.sh"));
        assert!(DOC.contains("test_run_cross_chain_outbound_intent_contract_lane.sh"));
    }

    #[test]
    fn doc_includes_bridge_adapter_conformance_contract() {
        assert!(DOC.contains("## Bridge Adapter Dry-Run Conformance Contract (Issue #907)"));
        assert!(DOC.contains("kamn.bridge.adapter-conformance.v1"));
        assert!(DOC.contains("kamn.bridge.adapter-conformance.matrix-report.v1"));
        assert!(DOC.contains("bridge_adapter_conformance_reason_codes:GO:v1"));
        assert!(DOC.contains("bridge_adapter_conformance_reason_codes:NO-GO:v1"));
        assert!(DOC.contains("Regression: #907"));
    }

    #[test]
    fn doc_includes_bridge_adapter_conformance_test_commands() {
        assert!(DOC.contains("bridge_adapter_conformance_contract.py"));
        assert!(DOC.contains("test_generate_bridge_adapter_conformance_evidence_bundle.sh"));
        assert!(DOC.contains("test_run_bridge_adapter_conformance_contract_lane.sh"));
    }
}

mod redaction_tombstones_docs {
    const DOC: &str = include_str!("../../../docs/foundation/redaction-tombstones.md");

    #[test]
    fn doc_contains_redaction_scope_and_validation() {
        assert!(DOC.contains("## Scope Delivered"));
        assert!(DOC.contains("RedactionComplianceEngine"));
        assert!(DOC.contains("## Local Validation"));
    }

    #[test]
    fn doc_contains_classification_redaction_compliance_contract_lane() {
        assert!(DOC.contains("## Classification/Redaction Compliance Contract Lane"));
        assert!(DOC.contains("classification_redaction_lane_contract.py"));
        assert!(DOC.contains("run_classification_redaction_contract_lane.sh"));
        assert!(DOC.contains("classification_redaction_contract_lane_contract.py"));
        assert!(DOC.contains("check_classification_redaction_policy.sh"));
        assert!(DOC.contains("classification_redaction_policy_contract.py"));
        assert!(DOC.contains("kamn.compliance.classification-redaction-report.v1"));
    }

    #[test]
    fn regression_requires_classification_redaction_fail_closed_marker() {
        // Regression: #914
        assert!(DOC.contains(
            "classification/redaction contract drift must fail closed (`Regression: #914`)."
        ));
    }

    #[test]
    fn regression_requires_classification_redaction_policy_wrapper_marker() {
        // Regression: #1222
        assert!(DOC.contains("classification_redaction_policy_contract.py"));
        assert!(DOC.contains("Regression: #1222"));
    }

    #[test]
    fn regression_requires_classification_redaction_lane_wrapper_marker() {
        // Regression: #1226
        assert!(DOC.contains("classification_redaction_lane_contract.py"));
        assert!(DOC.contains("Regression: #1226"));
    }

    #[test]
    fn regression_requires_classification_redaction_contract_lane_wrapper_marker() {
        // Regression: #1230
        assert!(DOC.contains("classification_redaction_contract_lane_contract.py"));
        assert!(DOC.contains("Regression: #1230"));
    }
}
