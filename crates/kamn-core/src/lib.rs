//! Core KAMN domain, protocol, and contract surfaces.
//!
//! This crate exports the public interfaces used by node, SDK, and contract-lane
//! tooling. Missing-docs lint is enabled in phased mode: legacy exports are
//! explicitly allow-listed while new public surfaces should carry docs by default.
#![warn(missing_docs)]

/// Agent key hierarchy roles, rotation, and ephemeral session key contracts.
pub mod agent_key_hierarchy;
/// Agent-driven upgrade proposal, review, and execution workflow contracts.
pub mod agent_upgrade_workflow;
/// Anti-spam admission, rate-limit, and suspension policy contracts.
pub mod anti_spam;
/// Audit export filters, bundles, and governance evidence contracts.
pub mod audit_exports;
/// Mempool block production and consensus-validation pipeline contracts.
pub mod block_pipeline;
pub mod bootstrap;
/// Bridge ingress and egress normalization plus policy evaluation contracts.
pub mod bridge_adapter;
/// Channel metadata, snapshot persistence, and recovery validation contracts.
pub mod channel_models;
/// Channel policy registration, membership authorization, and retention contracts.
pub mod channel_policies;
/// Node role, sync mode, and runtime configuration validation contracts.
pub mod config;
/// Content retention, tombstone scheduling, and purge eligibility contracts.
pub mod content_lifecycle;
/// Replication policy, availability health, and repair action contracts.
pub mod content_replication;
/// Retrieval access policy, caching controls, and audit event contracts.
pub mod content_retrieval;
/// Content storage adapter contracts, object metadata, and CID/URI helpers.
pub mod content_storage;
/// Cross-chain route validation, inbound normalization, and outbound quorum dispatch contracts.
pub mod cross_chain_bridge;
/// Cross-chain receipt proof normalization and finality mapping contracts.
pub mod cross_chain_receipt;
/// Data-domain classification policy and write-tag validation contracts.
pub mod data_classification;
/// M0 data-layer foundation records, append-only ledger, and hash-chain contracts.
pub mod data_layer_m0;
/// M1 trust-anchor contracts for merkle batching, proof APIs, and Kolme anchoring worker flows.
pub mod data_layer_m1;
/// M10 partition contracts for scaling controls, archival index, and re-attachment lifecycle.
pub mod data_layer_m10_partition_archival;
/// M11 closure evidence contracts for release acceptance decision synthesis.
pub mod data_layer_m11_closure_evidence;
/// M11 hardening contracts for security/chaos/performance matrix and operator readiness synthesis.
pub mod data_layer_m11_hardening_readiness;
/// M1 orchestrator contracts for scheduler + anchoring + persistence-plan projection.
pub mod data_layer_m1_anchoring_orchestrator;
/// M1 scheduler contracts for deterministic count/window trigger evaluation.
pub mod data_layer_m1_batch_scheduler;
/// M2 access-gateway contracts for DID authn/authz, RLS templates, and audit chains.
pub mod data_layer_m2_gateway_access;
/// M3 search contracts for owner-scoped blind-index and metadata query APIs.
pub mod data_layer_m3_blind_index_search;
/// M4 escrow integration contracts for state transitions, scoped messaging, and settlement evidence.
pub mod data_layer_m4_escrow_integration;
/// M5 vector-layer contracts for embedding ingestion, semantic retrieval, and anomaly scoring.
pub mod data_layer_m5_vector_integration;
/// M6 graph-layer contracts for owner-scoped schema, trust propagation, and portability.
pub mod data_layer_m6_graph_integration;
/// M7 time-series contracts for telemetry ingest, rollups, and owner billing projections.
pub mod data_layer_m7_timeseries_telemetry;
/// M8 compliance contracts for retention policy, legal hold, and crypto-shredding lifecycle.
pub mod data_layer_m8_compliance_lifecycle;
/// M9 gateway bridge contracts for deterministic transport-envelope projection.
pub mod data_layer_m9_gateway_bridge;
/// M9 realtime contracts for dispatch acknowledgements, scoped presence, and backpressure markers.
pub mod data_layer_m9_realtime_delivery;
/// Phase-2 operational pipeline contracts for envelope crypto + blind-index derivation.
pub mod data_layer_phase2_crypto_blind_index_pipeline;
/// PostgreSQL execution adapter contracts for live bridge-descriptor execution and migrations.
pub mod data_layer_postgres_execution_adapter;
/// PostgreSQL repository bridge contracts for deterministic data-layer SQL descriptor projection.
pub mod data_layer_postgres_repository_bridge;
/// PRD critical-scenario conformance contracts for shell-neutral validation (`62..71`).
pub mod data_layer_prd_critical_scenario_conformance;
/// Shell-neutral orchestration and shell/rust ratio-budget policy contracts.
pub mod data_layer_shell_neutral_policy;
/// DID document canonicalization and federated trust-handshake contracts.
pub mod did;
/// DID registry lifecycle and chain-submission finality contracts.
pub mod did_registry;
/// Direct-message encryption/decryption contracts and error semantics.
pub mod direct_message_crypto;
/// Discord bridge ingress/egress routing and outbound approval contracts.
pub mod discord_bridge;
/// Durable snapshot contracts for guard-state and policy persistence.
pub mod durable_guard_store;
/// Escrow hold, release, refund, and dispute lifecycle contracts.
pub mod escrow;
/// Fairness/starvation policy contracts for deterministic overload-governance checks.
pub mod fairness_policy;
/// Governance proposal, voting, and execution lifecycle contracts.
pub mod governance_workflow;
/// Group sender-key distribution, rotation, and encryption integrity contracts.
pub mod group_channel_crypto;
/// Instruction claim verification, inclusion-proof checks, and record contracts.
pub mod instruction_verify;
/// Invariant catalog, taxonomy, and guardrail policy contracts.
pub mod invariants;
/// Key rotation/revocation state machine and audit-trail verification contracts.
pub mod key_lifecycle;
/// Key compromise and recovery lifecycle contracts.
pub mod key_recovery;
pub mod kolme_runtime_commit;
/// Message delivery replay, nonce, and acceptance window guardrail contracts.
pub mod message_delivery_guards;
/// Canonical message envelope schema validation and normalization contracts.
pub mod message_envelope;
/// Message lifecycle models, snapshot contracts, and proof-admission flow.
pub mod message_lifecycle;
/// Message proof anchor submission contracts aligned to lifecycle transitions.
pub mod message_proof_anchoring;
/// State schema migration planning and validation contracts.
pub mod migrations;
pub mod namespaces;
/// Observability sampling, SLO projection, and report synthesis contracts.
pub mod observability;
/// Permissioned operator configuration actions and audit-log service contracts.
pub mod operator_actions;
/// Operator identity binding, proof validation, and authorization contracts.
pub mod operator_binding;
/// Operator dashboard API payload contracts and audit feed query validation.
pub mod operator_dashboard_api;
/// Operator dashboard UI composition contracts and presentation-ready projections.
pub mod operator_dashboard_ui;
/// Peer discovery and gossip transport adapter contracts for runtime lifecycle flows.
pub mod p2p_transport;
/// Performance target thresholds and benchmark outcome classification contracts.
pub mod performance_targets;
/// Per-scope quota policy evaluation and deterministic fail-closed taxonomy contracts.
pub mod quota_policy;
/// Redaction request approval, audit-event, and visibility compliance contracts.
pub mod redaction_compliance;
/// Reputation-signal weighting and candidate-ranking contracts for routing.
pub mod reputation_signals;
/// Reputation state persistence, restore, and export contracts.
pub mod reputation_state;
/// Retention policy evaluation, tombstone lifecycle, and purge guard contracts.
pub mod retention_engine;
/// Runtime lifecycle, queue, quorum, watchdog, and recovery contracts.
pub mod runtime;
/// Service marketplace listing registration, search, and negotiation hooks.
pub mod service_marketplace;
/// Signature-profile compatibility fixtures and baseline verification helpers.
pub mod signature_profile;
/// Signer backend routing, secure-provider policy, and signature validation contracts.
pub mod signer_backend;
/// Deterministic triadic runtime smoke simulation contracts.
pub mod smoke;
/// Legacy file snapshot to sqlite migration parity-check contracts.
pub mod snapshot_migration;
/// Sqlite backend bootstrap/versioning and namespace-key-value persistence contracts.
pub mod sqlite_store_backend;
pub mod state;
/// Task artifact registration, integrity checks, and lookup contracts.
pub mod task_artifacts;
pub mod task_lifecycle;
/// Task mutation APIs, dependency graph orchestration, and snapshot persistence contracts.
pub mod task_operations;
/// Task payment offer/confirmation workflow backed by escrow release controls.
pub mod task_payment;
/// Telegram inbound bridge validation, route checks, and envelope normalization.
pub mod telegram_bridge;
/// Token configuration, supply allocation, and transfer guard contracts.
pub mod token;
/// Baseline transaction validation and state-hash progression guard contracts.
pub mod transaction;
/// Trust score policy model, abuse penalties, and persistence calculation helpers.
pub mod trust_score;
/// Runtime upgrade proposal, rollout orchestration, and rollback policy contracts.
pub mod upgrade_orchestration;
/// Validator onboarding/offboarding, quorum-change, and rollback lifecycle contracts.
pub mod validator_lifecycle;
/// Runtime watchdog anomaly taxonomy and report contracts.
pub mod watchdog;
/// Zero-knowledge message-proof option evaluation, witness, and consensus contracts.
pub mod zk_message_proofs;

pub use agent_key_hierarchy::{
    AgentKeyHierarchy, AgentKeyHierarchyError, EphemeralSessionKey, KeyRole,
};
pub use agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEvent, AgentUpgradeAuditEventKind,
    AgentUpgradeProposalDraft, AgentUpgradeProposalRecord, AgentUpgradeProposalState,
    AgentUpgradeWorkflowConfig, AgentUpgradeWorkflowError,
};
pub use anti_spam::{
    AntiSpamConfig, AntiSpamDecision, AntiSpamEngine, AntiSpamError, AntiSpamRejection,
    AntiSpamTelemetry,
};
pub use audit_exports::{
    AuditDomain, AuditEventRecord, AuditExportBundle, AuditExportEngine, AuditExportError,
    AuditExportFilter, AuditExportFormat, AuditExportManifest, AuditExportRequest,
};
pub use block_pipeline::{
    build_canonical_replay_evidence_bundle, build_transport_convergence_evidence_bundle,
    decode_transport_candidate_payload, decode_transport_canonical_candidate_payload,
    durable_commit_checker_reason_taxonomy_version, encode_transport_candidate_payload,
    encode_transport_canonical_candidate_payload, encode_transport_commit_report_payload,
    enforce_durable_commit_checker_lane_boundary, project_durable_commit_checker_reason,
    AcceptAllForkChoiceHook, BlockConsensusRoundInput, BlockPipelineCommitReport,
    BlockPipelineError, CanonicalCandidateDecision, CanonicalCandidateOutcome,
    CanonicalCommitRecord, CanonicalCommitStore, CanonicalReplayEvidenceBundle,
    DeterministicCompetingBranchForkChoiceHook, DurableCommitCheckerLaneBoundaryReport,
    DurableCommitCheckerLaneMode, DurableCommitCheckerReasonClass,
    DurableCommitCheckerReasonProjection, FileCanonicalCommitStore, ForkChoiceDecision,
    ForkChoiceHook, GossipFrameTransportMempoolFeed, GossipIngressAdapter, GossipIngressBatch,
    GossipIngressError, GossipIngressRecord, InMemoryCanonicalCommitStore,
    InMemoryTransportMempoolFeed, MempoolBlockPipeline, SqliteCanonicalCommitStore,
    TransportCanonicalCandidateFeed, TransportConvergenceEvidenceBundle, TransportEventMempoolFeed,
    TransportFedBlockPipeline, TransportMempoolFeed,
};
pub use bootstrap::{
    bootstrap, bootstrap_from_state_version, bootstrap_with_transport_profile, BootstrapPlan,
};
pub use bridge_adapter::{
    AllowAllBridgePolicy, BridgeAdapter, BridgeAdapterEngine, BridgeAdapterError, BridgeDirection,
    BridgeInboundEnvelope, BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform,
    BridgePolicyHook, NormalizedInboundMessage, PassThroughBridgeAdapter,
};
pub use channel_models::{
    ChannelMetadata, ChannelModelError, ChannelRecordSnapshot, ChannelRecoveryResult,
    ChannelSnapshot, ChannelSnapshotError, ChannelSnapshotStore, ChannelSnapshotStoreError,
    ChannelStore, ChannelType, FileChannelSnapshotStore, InMemoryChannelSnapshotStore,
    SqliteChannelSnapshotStore, CHANNEL_SNAPSHOT_SCHEMA_VERSION,
};
pub use channel_policies::{
    ChannelAction, ChannelPermissionEngine, ChannelPermissions, ChannelPolicyError,
    ChannelPolicySnapshot, ChannelPolicySnapshotChannel, ChannelPolicySnapshotError,
    PermissionRule, RetentionMessage, RetentionPolicy, CHANNEL_POLICY_SNAPSHOT_SCHEMA_VERSION,
};
pub use config::{
    ConfigError, NodeConfig, NodeRole, SyncMode, SyncOperationalProfile, SyncRecoveryStrategy,
    SyncStartupStrategy,
};
pub use content_lifecycle::{
    ContentCleanupAction, ContentCleanupActionKind, ContentLifecycleError, ContentLifecycleManager,
    ContentLifecycleRecord, ContentLifecycleStatus, ContentRetentionClass, ContentRetentionProfile,
};
pub use content_replication::{
    ContentAvailabilityAlert, ContentAvailabilityHealth, ContentAvailabilitySnapshot,
    ContentRepairAction, ContentRepairReason, ContentReplicationError, ContentReplicationManager,
    ContentReplicationPolicy,
};
pub use content_retrieval::{
    ContentRetrievalAuditEvent, ContentRetrievalConfig, ContentRetrievalEngine,
    ContentRetrievalError, ContentRetrievalOutcome, ContentRetrievalRequest,
    ContentRetrievalResult, ContentRetrievalScope,
};
pub use content_storage::{
    cid_from_content_uri, content_uri_for_cid, ContentHead, ContentObject, ContentStorageAdapter,
    ContentStorageError, FileContentAdapter, InMemoryContentAdapter,
};
pub use cross_chain_bridge::{
    CrossChainBridgeConfig, CrossChainBridgeEngine, CrossChainBridgeError,
    CrossChainInboundRequest, CrossChainNetwork, CrossChainOutboundApproval,
    CrossChainOutboundDispatch,
};
pub use cross_chain_receipt::{
    normalize_cross_chain_receipt, CrossChainReceiptFinality, CrossChainReceiptNetwork,
    CrossChainReceiptNormalizationError, CrossChainReceiptProof, CrossChainReceiptStatus,
    NormalizedCrossChainReceipt, ETHEREUM_FINAL_CONFIRMATION_THRESHOLD,
};
pub use data_classification::{
    ClassificationPolicy, ClassificationStatus, DataClassificationEngine, DataClassificationError,
    DataClassificationLevel, WriteDomain, WriteRequestContext, WriteTag,
};
pub use data_layer_m0::{
    evaluate_data_layer_m0_conformance_matrix, DataLayerM0AppendOnlyLedger,
    DataLayerM0ConformanceInvariant, DataLayerM0ConformanceMatrixCase,
    DataLayerM0ConformanceMatrixDecision, DataLayerM0ConformanceMatrixEvidence,
    DataLayerM0ConformanceMatrixReport, DataLayerM0EnvelopeRecord, DataLayerM0Error,
    DataLayerM0RecordInput, DataLayerM0WrappedKey, DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD,
    DATA_LAYER_M0_CONFORMANCE_MATRIX_DRIFT_REASON_CODE,
    DATA_LAYER_M0_CONFORMANCE_MATRIX_STABLE_REASON_CODE, DATA_LAYER_M0_HASH_ALGORITHM,
    DATA_LAYER_M0_HASH_CHAIN_GENESIS,
};
pub use data_layer_m1::{
    evaluate_data_layer_m1_anchor_failure_matrix, evaluate_data_layer_m1_inclusion_proof,
    verify_data_layer_m1_inclusion_proof, DataLayerM1AnchorFailureMatrixCase,
    DataLayerM1AnchorFailureMatrixDecision, DataLayerM1AnchorFailureMatrixEvidence,
    DataLayerM1AnchorFailureMatrixReport, DataLayerM1AnchorOutcome, DataLayerM1AnchorOutcomeKind,
    DataLayerM1AnchorReceipt, DataLayerM1AnchorResult, DataLayerM1AnchorRetryClass,
    DataLayerM1Error, DataLayerM1KolmeAnchoringWorker, DataLayerM1MerkleBatch,
    DataLayerM1MerkleInclusionProof, DataLayerM1MerkleLeaf, DataLayerM1MerkleProofStep,
    DataLayerM1ProofSiblingSide, DataLayerM1ProofVerificationDecision,
    DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
    DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE, DATA_LAYER_M1_HASH_ALGORITHM,
    DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
    DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
};
pub use data_layer_m10_partition_archival::{
    data_layer_m10_format_partition_name, data_layer_m10_project_archival_retry_decision,
    DataLayerM10ArchivalFailureClass, DataLayerM10ArchivalIndexEntry,
    DataLayerM10ArchivalRecoveryAction, DataLayerM10ArchivalRetryDecision,
    DataLayerM10ArchivalRetryPolicy, DataLayerM10ArchiveDueRequest,
    DataLayerM10ComplianceShredProjectionReport, DataLayerM10ComplianceShredProjectionRequest,
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10PartitionRecord, DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus,
    DataLayerM10RecoveryDecision, DataLayerM10RecoveryReadinessReport,
    DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
    DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
    DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD, DATA_LAYER_M10_ARCHIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
    DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE, DATA_LAYER_M10_PARTITION_PREFIX,
    DATA_LAYER_M10_REATTACH_REASON_CODE, DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
};
pub use data_layer_m11_closure_evidence::{
    data_layer_m11_evaluate_closure_evidence, DataLayerM11ClosureAcceptanceDecision,
    DataLayerM11ClosureEvidenceError, DataLayerM11ClosureEvidenceInput,
    DataLayerM11ClosureEvidenceReport, DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
};
pub use data_layer_m11_hardening_readiness::{
    DataLayerM11HardeningMatrix, DataLayerM11HardeningMatrixError,
    DataLayerM11OperatorReadinessDecision, DataLayerM11OperatorReadinessReport,
    DataLayerM11ScenarioDefinition, DataLayerM11ScenarioDomain, DataLayerM11ScenarioOutcomeInput,
    DataLayerM11ScenarioOutcomeRecord, DataLayerM11ScenarioSeverity, DataLayerM11ScenarioStatus,
    DATA_LAYER_M11_BLOCK_CRITICAL_FAILURE_REASON_CODE,
    DATA_LAYER_M11_BLOCK_REQUIRED_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M11_INVALID_TRANSITION_REASON_CODE, DATA_LAYER_M11_READINESS_GO_REASON_CODE,
};
pub use data_layer_m1_anchoring_orchestrator::{
    reconcile_data_layer_m1_finality_observation, DataLayerM1AnchoringConfirmationMetadata,
    DataLayerM1AnchoringFinalityObservation, DataLayerM1AnchoringFinalityReconciliationProjection,
    DataLayerM1AnchoringFollowUpAction, DataLayerM1AnchoringFollowUpPolicy,
    DataLayerM1AnchoringMessageAssignment, DataLayerM1AnchoringOrchestrator,
    DataLayerM1AnchoringOrchestratorError, DataLayerM1AnchoringPersistencePlan,
    DataLayerM1AnchoringSubmissionMetadata, DataLayerM1AnchoringTickOutcome,
    DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_FINAL_BLOCK_HEIGHT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_TX_MISMATCH_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_CONFLICT_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FAILED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FINAL_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_RETRY_IN_FLIGHT_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_DEFERRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_REJECTED_REASON_CODE,
};
pub use data_layer_m1_batch_scheduler::{
    evaluate_data_layer_m1_batch_trigger, DataLayerM1BatchSchedulerError,
    DataLayerM1BatchSchedulerPolicy, DataLayerM1BatchTriggerDecision,
    DataLayerM1PendingBatchMessage, DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_COUNT_THRESHOLD,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_DEFERRED,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_WINDOW_THRESHOLD,
};
pub use data_layer_m2_gateway_access::{
    data_layer_m2_default_rls_policies, DataLayerM2AbacEngine, DataLayerM2AccessAuditInput,
    DataLayerM2AccessAuditLedger, DataLayerM2AccessAuditRecord, DataLayerM2ActorRole,
    DataLayerM2AuthorizationDecision, DataLayerM2DidAuthRequest,
    DataLayerM2DidAuthRequestValidated, DataLayerM2DidSessionService, DataLayerM2GatewayError,
    DataLayerM2MessageScope, DataLayerM2MessageScopeValidated,
    DataLayerM2NegativeAuthorizationAuditFixture, DataLayerM2NegativeAuthorizationCase,
    DataLayerM2NegativeAuthorizationMatrixDecision, DataLayerM2NegativeAuthorizationMatrixReport,
    DataLayerM2RlsPolicy, DataLayerM2SessionToken, DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS,
    DATA_LAYER_M2_HASH_ALGORITHM, DATA_LAYER_M2_INVALID_RECIPIENT_DID_REASON_CODE,
    DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE, DATA_LAYER_M2_INVALID_SENDER_DID_REASON_CODE,
    DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
    DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
    DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED, DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
    DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED, DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
    DATA_LAYER_M2_REQUESTER_DID_SETTING,
};
pub use data_layer_m3_blind_index_search::{
    data_layer_m3_compute_blind_index, data_layer_m3_normalize_blind_index_value,
    DataLayerM3BlindIndexDeterminismDecision, DataLayerM3BlindIndexDeterminismInput,
    DataLayerM3BlindIndexDeterminismReport, DataLayerM3BlindIndexQuery,
    DataLayerM3BlindIndexRetrievalProjectionInput, DataLayerM3BlindIndexSearchMode,
    DataLayerM3MessageMetadataRecord, DataLayerM3MetadataQuery,
    DataLayerM3RetrievalProjectionRecord, DataLayerM3SearchCatalog, DataLayerM3SearchError,
    DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE,
    DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE,
    DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE, DATA_LAYER_M3_HASH_ALGORITHM,
};
pub use data_layer_m4_escrow_integration::{
    DataLayerM4EscrowDraftInput, DataLayerM4EscrowInteropError, DataLayerM4EscrowRecord,
    DataLayerM4EscrowState, DataLayerM4EscrowTransitionAction, DataLayerM4EscrowTransitionEngine,
    DataLayerM4EscrowTransitionEvidence, DataLayerM4EscrowVisibilityDecision,
    DataLayerM4EscrowVisibilityRequest, DataLayerM4SettlementEvidenceInput,
    DataLayerM4SettlementEvidenceReconciliationDecision,
    DataLayerM4SettlementEvidenceReconciliationReport, DataLayerM4SettlementEvidenceRecord,
    DataLayerM4SettlementEvidenceRegistry, DataLayerM4SettlementEvidenceRegistryError,
    DATA_LAYER_M4_ESCROW_ACTIVE_REASON_CODE,
    DATA_LAYER_M4_ESCROW_AUDITOR_DISPUTE_REQUIRED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_AUDITOR_SCOPE_ALLOWED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_CONFIGURED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_AUDITOR_THRESHOLD_NOT_MET_REASON_CODE,
    DATA_LAYER_M4_ESCROW_DISPUTED_REASON_CODE, DATA_LAYER_M4_ESCROW_EXPIRED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_FUNDED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_PARTICIPANT_SCOPE_ALLOWED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_REFUNDED_REASON_CODE, DATA_LAYER_M4_ESCROW_RELEASED_REASON_CODE,
    DATA_LAYER_M4_ESCROW_SCOPE_DENIED_REASON_CODE, DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS,
    DATA_LAYER_M4_HASH_ALGORITHM,
    DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M4_SETTLEMENT_EVIDENCE_RECONCILIATION_MISMATCH_REASON_CODE,
};
pub use data_layer_m5_vector_integration::{
    DataLayerM5AnomalyDecision, DataLayerM5AnomalyEvaluationInput, DataLayerM5EmbeddingPrivacyMode,
    DataLayerM5EmbeddingRecord, DataLayerM5EmbeddingRecordInput, DataLayerM5EmbeddingRegistry,
    DataLayerM5RecallDriftDecision, DataLayerM5RecallDriftEvaluationInput,
    DataLayerM5RecallDriftReport, DataLayerM5RetentionDueCandidate, DataLayerM5SemanticQuery,
    DataLayerM5SemanticQueryResult, DataLayerM5VectorIntegrationError,
    DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
    DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE, DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS,
    DATA_LAYER_M5_HASH_ALGORITHM, DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
    DATA_LAYER_M5_OWNER_SIDE_ANOMALY_REQUIRES_LOCAL_PIPELINE_REASON_CODE,
    DATA_LAYER_M5_OWNER_SIDE_PLAINTEXT_STORAGE_NOT_ALLOWED_REASON_CODE,
    DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
    DATA_LAYER_M5_PLAINTEXT_INDEX_MISSING_FOR_OWNER_SCOPE_REASON_CODE,
    DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE, DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE,
    DATA_LAYER_M5_RETENTION_DUE_REASON_CODE,
    DATA_LAYER_M5_SERVER_SIDE_PLAINTEXT_REQUIRED_REASON_CODE,
    DATA_LAYER_M5_VECTOR_DISTANCE_METRIC_COSINE,
};
pub use data_layer_m6_graph_integration::{
    DataLayerM6GraphEdgeInput, DataLayerM6GraphEdgeRecord, DataLayerM6GraphEdgeRelation,
    DataLayerM6GraphIntegrationError, DataLayerM6GraphNodeInput, DataLayerM6GraphNodeKind,
    DataLayerM6GraphNodeRecord, DataLayerM6GraphRegistry, DataLayerM6PortableEdgeProjection,
    DataLayerM6TrustPropagationQuery, DataLayerM6TrustPropagationResult,
    DATA_LAYER_M6_CROSS_OWNER_EDGE_DENIED_REASON_CODE, DATA_LAYER_M6_GRAPH_ENGINE_APACHE_AGE,
    DATA_LAYER_M6_GRAPH_PORTABILITY_PROFILE, DATA_LAYER_M6_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M6_TRUST_PROPAGATION_REASON_RANKED,
};
pub use data_layer_m7_timeseries_telemetry::{
    data_layer_m7_project_observability_sample, DataLayerM7AgentDailyAggregate,
    DataLayerM7AgentHourlyAggregate, DataLayerM7BillingQuery,
    DataLayerM7BillingReconciliationDecision, DataLayerM7BillingReconciliationInput,
    DataLayerM7BillingReconciliationReport, DataLayerM7NetworkHourlyAggregate,
    DataLayerM7OwnerBillingDailyProjection, DataLayerM7OwnerObservabilityReport,
    DataLayerM7TelemetryPointInput, DataLayerM7TelemetryPointRecord, DataLayerM7TelemetryRegistry,
    DataLayerM7TelemetryScopeQuery, DataLayerM7TimeseriesError,
    DATA_LAYER_M7_AGGREGATE_REASON_CODE, DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE, DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
    DATA_LAYER_M7_HOURLY_BUCKET_SECONDS, DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE,
    DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
};
pub use data_layer_m8_compliance_lifecycle::{
    data_layer_m8_retention_window_aligned_with_content_lifecycle,
    data_layer_m8_retention_window_seconds, DataLayerM8ComplianceError,
    DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest, DataLayerM8LegalHoldRequest,
    DataLayerM8MessageRecord, DataLayerM8MessageRecordInput, DataLayerM8OwnerScopeQuery,
    DataLayerM8RetentionClass, DataLayerM8RetentionDueCandidate, DataLayerM8RetentionInteropError,
    DataLayerM8WrappedCekInput, DATA_LAYER_M8_CEK_TOMBSTONE_MARKER,
    DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE, DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS,
    DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS, DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M8_RETENTION_DUE_REASON_CODE, DATA_LAYER_M8_STANDARD_RETENTION_SECONDS,
};
pub use data_layer_m9_gateway_bridge::{
    data_layer_m9_gateway_project_dispatch_event, data_layer_m9_gateway_project_presence_event,
    DataLayerM9GatewayBridgeError, DataLayerM9GatewayDispatchProjection,
    DataLayerM9GatewayDispatchProjectionRequest, DataLayerM9GatewayPresenceProjection,
    DataLayerM9GatewayPresenceProjectionRequest, DataLayerM9GatewayTransportProfile,
    DATA_LAYER_M9_GATEWAY_DISPATCH_EVENT_LABEL, DATA_LAYER_M9_GATEWAY_PRESENCE_EVENT_LABEL,
    DATA_LAYER_M9_GATEWAY_PRESENCE_NOT_FOUND_REASON_CODE,
    DATA_LAYER_M9_GATEWAY_PRESENCE_VISIBLE_REASON_CODE,
    DATA_LAYER_M9_GATEWAY_UNSUPPORTED_TRANSPORT_REASON_CODE,
};
pub use data_layer_m9_realtime_delivery::{
    DataLayerM9ChannelDispatchAuthorizationRequest, DataLayerM9DispatchAckStatus,
    DataLayerM9DispatchOutcome, DataLayerM9DispatchRequest, DataLayerM9PresenceConnectRequest,
    DataLayerM9PresenceQuery, DataLayerM9PresenceRecord, DataLayerM9PresenceRelationshipRequest,
    DataLayerM9RealtimeDeliveryError, DataLayerM9RealtimeDeliveryRegistry,
    DataLayerM9RecipientQueueSnapshot, DataLayerM9RuntimeBackpressureProjection,
    DataLayerM9RuntimeBackpressureProjectionRequest, DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE,
    DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE, DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
    DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE,
    DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS,
    DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS,
    DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
    DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
    DATA_LAYER_M9_INVALID_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_OWNER_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_TARGET_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES, DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
    DATA_LAYER_M9_RUNTIME_BACKPRESSURE_EVALUATION_FAILED_REASON_CODE,
    DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE,
};
pub use data_layer_phase2_crypto_blind_index_pipeline::{
    data_layer_phase2_build_operational_artifact, DataLayerPhase2OperationalPipelineArtifact,
    DataLayerPhase2OperationalPipelineError, DataLayerPhase2OperationalPipelineRequest,
    DataLayerPhase2RecipientEncryptionBinding,
};
pub use data_layer_postgres_execution_adapter::{
    data_layer_pg_collect_migration_files, DataLayerPgBlindIndexSearchRow,
    DataLayerPgExecutionAdapter, DataLayerPgExecutionAdapterConfig,
    DataLayerPgExecutionAdapterError, DataLayerPgMigrationReport, DataLayerPgRlsApplyReport,
    DataLayerPgRlsStatementOutcome, DataLayerPgStoredMessage,
    DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE,
    DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
    DATA_LAYER_PG_EXECUTION_SESSION_FAILED_REASON_CODE,
    DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
};
pub use data_layer_postgres_repository_bridge::{
    data_layer_pg_project_blind_index_search_operation,
    data_layer_pg_project_default_rls_statements, data_layer_pg_project_insert_message_operation,
    data_layer_pg_project_m5_embedding_insert_operation,
    data_layer_pg_project_m5_similarity_search_operation,
    data_layer_pg_project_m6_age_edge_upsert_operation,
    data_layer_pg_project_m6_age_trust_query_operation,
    data_layer_pg_project_m7_timescale_ingest_operation,
    data_layer_pg_project_m7_timescale_owner_rollup_query_operation,
    data_layer_pg_project_select_message_by_id_operation, DataLayerPgBlindIndexSearchRequest,
    DataLayerPgM5PgvectorConfig, DataLayerPgM5SimilaritySearchRequest, DataLayerPgM6AgeConfig,
    DataLayerPgM6AgeTrustQueryRequest, DataLayerPgM7TimescaleConfig,
    DataLayerPgM7TimescaleOwnerRollupRequest, DataLayerPgOperationKind,
    DataLayerPgRepositoryBridgeError, DataLayerPgRequesterSession, DataLayerPgRlsStatement,
    DataLayerPgSqlOperation, DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
    DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE, DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
};
pub use data_layer_prd_critical_scenario_conformance::{
    DataLayerPrdCriticalScenarioConformanceDecision, DataLayerPrdCriticalScenarioConformanceError,
    DataLayerPrdCriticalScenarioConformanceMatrix, DataLayerPrdCriticalScenarioConformanceReport,
    DataLayerPrdCriticalScenarioMode, DataLayerPrdCriticalScenarioResultInput,
    DataLayerPrdCriticalScenarioResultRecord,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE,
};
pub use data_layer_shell_neutral_policy::{
    data_layer_evaluate_shell_neutral_policy, DataLayerShellNeutralPolicyDecision,
    DataLayerShellNeutralPolicyError, DataLayerShellNeutralPolicyInput,
    DataLayerShellNeutralPolicyReport,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_VERIFIED_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON_CODE,
};
pub use did::{
    canonical_did_document, canonical_service_endpoint,
    validate_did_verification_method_algorithms, AgentDid, AgentDidError, AgentDidMetadata,
    DidDocument, DidDocumentError, DidService, DidVerificationMethod,
    FederatedDidHandshakeDecision, FederatedDidHandshakeError, FederatedDidHandshakeEvaluator,
    FederatedDidHandshakeInput, FederatedDidTrustStore, InMemoryFederatedDidTrustStore, KamnDid,
    KamnDidError,
};
pub use did_registry::{
    DidChainSubmissionOutcome, DidChainSubmissionReceipt, DidChainSubmissionRequest,
    DidChainSubmissionResult, DidLifecycleChainAdapter, DidLifecycleChainSubmissionRequest,
    DidLifecycleChainSubmissionResult, DidLifecycleMutationAction, DidLifecycleMutationEvidence,
    DidLifecycleMutationRequest, DidRegistrationChainAdapter, DidRegistry, DidRegistryError,
    DidSubmissionFinalityRecord, DidSubmissionFinalityStatus, DidSubmissionRetryClass,
    FileDidRegistrationChainAdapter, InMemoryDidRegistrationChainAdapter,
    KolmeDidLifecycleChainAdapter,
};
pub use direct_message_crypto::{
    DirectMessageCiphertext, DirectMessageCryptoEngine, DirectMessageCryptoError,
    DIRECT_MESSAGE_CIPHER_ALGORITHM, DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
pub use discord_bridge::{
    DiscordBridgeConfig, DiscordBridgeEngine, DiscordBridgeError, DiscordInboundRequest,
    DiscordOutboundApproval, DiscordOutboundDispatch,
};
pub use durable_guard_store::{
    ChannelPolicySnapshotStore, DeliveryGuardSnapshotStore, DurableGuardBundleSnapshotStore,
    DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError, FileDurableGuardSnapshotStore,
    InMemoryDurableGuardSnapshotStore, SqliteDurableGuardSnapshotStore,
    DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
};
pub use escrow::{
    EscrowLifecycle, EscrowLifecycleError, EscrowReceiptFinality, EscrowSettlementAction,
    EscrowSettlementOutcome, EscrowStatus, EscrowTransitionAction, EscrowTransitionEvidence,
};
pub use fairness_policy::{
    evaluate_fairness_policy, fairness_policy_reason_codes_csv,
    fairness_policy_reason_taxonomy_version, FairnessPolicyDecision, FairnessPolicyInput,
    FairnessPolicyViolationReason,
};
pub use governance_workflow::{
    GovernanceExecutionRecord, GovernanceParameterChangeDraft, GovernanceProposalDraft,
    GovernanceProposalRecord, GovernanceProposalStatus, GovernanceVoteChoice, GovernanceVoteRecord,
    GovernanceWorkflow, GovernanceWorkflowError,
};
pub use group_channel_crypto::{
    GroupChannelCryptoEngine, GroupChannelCryptoError, GroupMessageCiphertext,
    SenderKeyDistributionRecord, GROUP_MESSAGE_CIPHER_ALGORITHM,
    GROUP_MESSAGE_KEY_DERIVATION_ALGORITHM,
};
pub use instruction_verify::{
    InstructionClaim, InstructionRecord, InstructionVerifier, VerificationContext,
    VerificationFailure, VerificationOutcome, DEFAULT_MAX_CLAIM_VALIDITY_WINDOW_SECS,
};
pub use invariants::{
    catalog as invariant_catalog, classify_smoke_error, classify_transaction_guard_error,
    invariant_by_id, validate_catalog, InvariantCatalogError, InvariantDomain,
    InvariantFailureCode, InvariantSpec, InvariantViolation,
};
pub use key_lifecycle::{
    KeyLifecycle, KeyLifecycleAuditError, KeyLifecycleAuditRecord, KeyLifecycleError,
    KeyLifecycleEvent, KeyLifecycleState,
};
pub use key_recovery::{KeyRecoveryManager, RecoveryError, RecoveryState};
pub use kolme_runtime_commit::{
    AdapterBackedKolmeRuntimeCommitClient, InMemoryKolmeRuntimeCommitClient,
    KolmeApiBroadcastRequest, KolmeApiBroadcastResponse, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse, KolmeCommitReceiptFinality,
    KolmeRuntimeCommitBlockFallbackReconciler, KolmeRuntimeCommitBlockFallbackTransport,
    KolmeRuntimeCommitClient, KolmeRuntimeCommitError, KolmeRuntimeCommitFinalityChecker,
    KolmeRuntimeCommitFinalityTransport, KolmeRuntimeCommitForkFinalityResolver,
    KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitLiveProvider,
    KolmeRuntimeCommitNotificationEvent, KolmeRuntimeCommitNotificationsConnection,
    KolmeRuntimeCommitNotificationsConnector, KolmeRuntimeCommitNotificationsConsumer,
    KolmeRuntimeCommitOutcome, KolmeRuntimeCommitProvider, KolmeRuntimeCommitProviderError,
    KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderReceipt,
    KolmeRuntimeCommitProviderTransport, KolmeRuntimeCommitReceipt, KolmeRuntimeCommitRequest,
    KolmeRuntimeCommitSignedBroadcastEnvelope, KolmeRuntimeCommitTransportErrorKind,
    KolmeRuntimeCommitWebsocketConnector, RuntimeCommitFinalityProjection,
    RuntimeCommitLifecycleRecord, RuntimeCommitLifecycleState, RuntimeCommitPipeline,
};
pub use message_delivery_guards::{
    DeliveryFailureCode, DeliveryGuardInput, DeliveryGuardSnapshot, DeliveryGuardSnapshotError,
    DeliveryValidationResult, FailedDeliveryNotice, MessageDeliveryGuards,
    DELIVERY_GUARD_SNAPSHOT_SCHEMA_VERSION,
};
pub use message_envelope::{
    AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata,
    EnvelopeProof, MessageEnvelopeError, CANONICAL_ENCRYPTION_ALGORITHM,
    CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE,
};
pub use message_lifecycle::{
    FileMessageLifecycleSnapshotStore, InMemoryMessageLifecycleSnapshotStore,
    MessageLifecycleError, MessageLifecycleRecoveryResult, MessageLifecycleSnapshot,
    MessageLifecycleSnapshotError, MessageLifecycleSnapshotStore,
    MessageLifecycleSnapshotStoreError, MessageLifecycleStore, MessageProofAdmissionError,
    MessageRecordSnapshot, MessageStatus, SqliteMessageLifecycleSnapshotStore,
    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
};
pub use message_proof_anchoring::{
    InMemoryMessageProofChainAdapter, KolmeMessageProofChainAdapter,
    MessageProofAnchorFinalityRecord, MessageProofAnchorFinalityStatus, MessageProofAnchorReceipt,
    MessageProofAnchorRequest, MessageProofAnchorResult, MessageProofAnchorRetryClass,
    MessageProofAnchorSubmissionOutcome, MessageProofAnchorSubmissionRequest,
    MessageProofAnchoringError, MessageProofAnchoringService, MessageProofChainAdapter,
};
pub use migrations::{MigrationPlan, MigrationRegistry, MigrationStep};
pub use namespaces::StateNamespaces;
pub use observability::{
    ObservabilityAlert, ObservabilityError, ObservabilityHealth, ObservabilityMetric,
    ObservabilityMonitor, ObservabilityReport, ObservabilitySample, ObservabilitySeverity,
    ObservabilitySloProfile, ObservabilitySnapshot,
};
pub use operator_actions::{
    OperatorActionAuditRecord, OperatorActionOutcome, OperatorActionServiceError,
    PermissionedOperatorActionService,
};
pub use operator_binding::{
    OperatorBindingAction, OperatorBindingEngine, OperatorBindingError, OperatorBindingProof,
    OperatorBindingRecord,
};
pub use operator_dashboard_api::{
    DashboardPage, DashboardPageRequest, OperatorAgentView, OperatorDashboardApi,
    OperatorDashboardApiError, OperatorDashboardSnapshot, OperatorEscrowView, OperatorMessageView,
    OperatorReputationView, OperatorTaskView,
};
pub use operator_dashboard_ui::{
    DashboardAttentionLevel, DashboardSummary, OperatorAgentListRow, OperatorAuditTraceEntry,
    OperatorDashboardUi, OperatorDashboardUiError, OperatorDashboardUiModel,
    OperatorEscrowStatusEntry, OperatorMessageTraceEntry, OperatorReputationOverviewEntry,
    OperatorTaskTimelineEntry, ReputationRiskTier,
};
pub use p2p_transport::{
    build_libp2p_lifecycle_regression_corpus, build_p2p_swarm_deterministic_config,
    canonical_libp2p_identify_protocol_id, canonical_libp2p_topic_id,
    compose_kademlia_discovery_bootstrap, compose_libp2p_swarm_behavior_stack,
    deterministic_multi_process_peer_validation_hooks, peer_adapter_reason_taxonomy_version,
    project_live_transport_reconnect_reason, project_peer_adapter_error_reason,
    resolve_libp2p_live_runtime_backend, run_libp2p_lifecycle_regression_case,
    run_libp2p_lifecycle_regression_corpus, InMemoryPeerLifecycleTransport,
    KademliaBootstrapSeedSet, KademliaDiscoveryBootstrapPlan, Libp2pBehaviorFailureClass,
    Libp2pLivePeerLifecycleTransport, Libp2pLiveRuntimeBackend, Libp2pRuntimeAdapterOperation,
    Libp2pRuntimeEvent, Libp2pRuntimeEventKind, LiveTransportFaultClass,
    LiveTransportReconnectDecision, LiveTransportReconnectPolicy, P2pSwarmBehaviorStack,
    P2pSwarmDeterministicConfig, P2pSwarmHarnessMode, P2pSwarmHarnessReport, P2pSwarmHarnessTask,
    P2pTransportError, PeerAdapterMultiProcessValidationHook, PeerAdapterReasonClass,
    PeerAdapterReasonProjection, PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleRegressionCase,
    PeerLifecycleRegressionError, PeerLifecycleRegressionExpectedOutcome,
    PeerLifecycleRegressionOutcome, PeerLifecycleTransport, PeerLifecycleTransportCoordinator,
    UdpPeerLifecycleTransport,
};
pub use performance_targets::{
    evaluate_performance_from_observability, evaluate_performance_run, PerformanceAggregate,
    PerformanceMetric, PerformanceMetricResult, PerformanceRunError, PerformanceRunReport,
    PerformanceSample, PrdPerformanceTargets,
};
pub use quota_policy::{
    evaluate_quota_policy, quota_policy_reason_codes_csv, quota_policy_reason_taxonomy_version,
    QuotaPolicyDecision, QuotaPolicyInput, QuotaPolicyViolationReason,
};
pub use redaction_compliance::{
    RedactionAction, RedactionAuditEvent, RedactionAuditEventKind, RedactionComplianceEngine,
    RedactionComplianceError, RedactionRequestStatus, RedactionVisibility,
};
pub use reputation_signals::{
    rank_agents_for_routing, rank_listings_by_reputation, RankedAgentCandidate,
    RankedListingCandidate, ReputationSignalError, ReputationSignalSummary, RoutingSignalWeights,
};
pub use reputation_state::{
    agent_state_key, AgentReputation, CapabilityVerification, DisputeRecord, Endorsement,
    ReputationError, ReputationPersistedRecord, ReputationStore, ReputationTaskOutcome,
    ScoreSnapshot, DEFAULT_TRUST_SCORE, MAX_TRUST_SCORE,
};
pub use retention_engine::{
    RetentionClass, RetentionDomain, RetentionEnginePolicy, RetentionEvaluation,
    RetentionPolicyEngine, RetentionPolicyError, RetentionRecord, RetentionStatus,
};
pub use runtime::{
    build_runtime_wiring, build_runtime_wiring_with_transport_profile, libp2p_feature_gate_name,
    resolve_libp2p_compile_mode, simulate_daemon_network_fault, AuthenticatedPeerFrame,
    AuthenticatedPeerFrameError, BoundedRuntimeQueue, DeterministicBackpressureController,
    DeterministicNetworkFaultSimulator, DeterministicProposalPlanner, FileRuntimeSnapshotStore,
    InMemoryRuntimeSnapshotStore, Libp2pCompileMode, NetworkFaultSimulationError,
    NetworkFaultSimulationInput, NetworkFaultSimulationReport, PeerFrameAuthenticator,
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, ProposalPlan,
    ProposalPlannerError, RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
    RuntimeBackpressureAction, RuntimeBackpressureDecision, RuntimeBackpressureError,
    RuntimeBackpressureInput, RuntimeBackpressurePolicy, RuntimeLifecycleError, RuntimeQueueError,
    RuntimeSnapshot, RuntimeSnapshotStore, RuntimeTransportProfile, RuntimeWiring,
    SnapshotRecoveryResult, SnapshotRestoreError, SnapshotRestoreGuard, SnapshotStoreError,
    SqliteRuntimeSnapshotStore,
};
pub use service_marketplace::{
    MarketplaceSearchFilter, NegotiationThreadHook, ServiceListing, ServiceMarketplaceEngine,
    ServiceMarketplaceError,
};
pub use signature_profile::{
    baseline_signature_algorithm, baseline_signature_for_fields, baseline_signature_profile_id,
    legacy_signature_for_fields, parse_signature_profile_metadata,
    signature_matches_supported_profile_for_fields,
    signature_profile_compatibility_fixtures_for_fields, unknown_signature_algorithm_for_fields,
    unknown_signature_profile_for_fields, SignatureProfileCompatibilityFixture,
    SignatureProfileMetadata, BASELINE_SIGNATURE_ALGORITHM, BASELINE_SIGNATURE_PROFILE_ID,
    LEGACY_SIGNATURE_PROFILE_ID, UNKNOWN_SIGNATURE_ALGORITHM_ID,
};
pub use signer_backend::{
    BackendSignature, LocalSignerBackend, SecureSignerBackend, SecureSignerProvider, SignerBackend,
    SignerBackendError, SignerBackendRouter, SignerKeyRole, SignerProviderHandshakeMatrix,
    SignerProviderHandshakeStatus, SigningRequest,
};
pub use smoke::{ProducedBlock, RoleSmokeNetwork, SmokeError};
pub use snapshot_migration::{
    migrate_file_snapshots_to_sqlite_parity, SnapshotMigrationError, SnapshotMigrationParityReport,
};
pub use sqlite_store_backend::{
    SqliteStoreBackend, SqliteStoreBackendError, SQLITE_STORE_SCHEMA_VERSION,
};
pub use state::{
    canonical_state_key, AppStateSchema, StateKeyError, StateVersion, APP_STATE_VERSION,
};
pub use task_artifacts::{
    TaskArtifactError, TaskArtifactRecord, TaskArtifactRegistry, TaskArtifactSubmission,
};
pub use task_lifecycle::{
    TaskLifecycle, TaskLifecycleError, TaskState, TaskTransition, TaskTransitionEvidence,
};
pub use task_operations::{
    FileTaskOperationSnapshotStore, InMemoryTaskOperationSnapshotStore,
    SqliteTaskOperationSnapshotStore, SwarmTaskDraft, TaskOperationEngine, TaskOperationError,
    TaskOperationNoticeKind, TaskOperationRecord, TaskOperationRecordSnapshot,
    TaskOperationRecoveryResult, TaskOperationSnapshot, TaskOperationSnapshotStore,
    TaskOperationSnapshotStoreError, TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
};
pub use task_payment::{PaymentConfirm, PaymentOffer, TaskPaymentError, TaskPaymentWorkflow};
pub use telegram_bridge::{
    TelegramBridgeConfig, TelegramBridgeEngine, TelegramBridgeError, TelegramInboundRequest,
};
pub use token::{
    default_token_config, AllocationBucket, GenesisAllocation, TokenConfig, TokenConfigError,
    DEFAULT_DECIMALS, DEFAULT_TOKEN_SYMBOL, DEFAULT_TOTAL_SUPPLY,
};
pub use transaction::{
    BaselineTransaction, TransactionGuardError, TransactionGuards, GENESIS_STATE_HASH,
};
pub use trust_score::{
    calculate_trust_score, recalculate_and_persist_trust_score, AbusePenaltyKind,
    TrustScoreBreakdown, TrustScoreError, TRUST_SCORE_ENGINE_VERSION, TRUST_SCORE_MAX,
    TRUST_SCORE_MIN,
};
pub use upgrade_orchestration::{
    UpgradeAuditEvent, UpgradeAuditEventKind, UpgradeOrchestrationError, UpgradeProposalRecord,
    UpgradeProposalState, VersionUpgradeAuditView, VersionUpgradeOrchestrator,
};
pub use validator_lifecycle::{
    ValidatorLifecycleError, ValidatorLifecycleManager, ValidatorSetSnapshot,
    ValidatorTransitionKind, ValidatorTransitionProof, ValidatorTransitionRecord,
};
pub use watchdog::{
    WatchdogAlert, WatchdogAlertKind, WatchdogConfig, WatchdogError, WatchdogNode,
    WatchdogObservation, WatchdogSeverity, WatchdogSnapshot,
};
pub use zk_message_proofs::{
    build_message_witness, evaluate_zk_option, phase4_baseline_options, recommend_phase4_plan,
    ProcessorProofAdmissionDecision, ProcessorProofAdmissionEvaluator,
    ProcessorProofAdmissionInput, ProcessorProofArtifact, ProofWatchdogProjection,
    ProofWatchdogProjectionKind, ProofWatchdogProjector, ProofWatchdogSeverity,
    ValidatorProofAttestation, ValidatorProofConsensusDecision, ValidatorProofConsensusError,
    ValidatorProofConsensusEvaluator, ValidatorProofConsensusInput, ValidatorProofConsensusStatus,
    ValidatorProofVerdict, ZkArchitectureOption, ZkDesignError, ZkEvaluationPolicy,
    ZkMessageWitness, ZkOptionAssessment, ZkPhaseMilestone, ZkPhasePlan, ZkProofSystem, ZkRisk,
    ZkRiskSeverity, ZkVerificationTopology,
};
