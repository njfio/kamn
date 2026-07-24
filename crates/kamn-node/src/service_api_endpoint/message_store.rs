use super::state_io::{load_service_api_state_payload, persist_service_api_state_payload};
use super::*;
use kamn_core::{
    data_layer_m11_evaluate_closure_evidence, data_layer_m3_compute_blind_index,
    verify_data_layer_m1_inclusion_proof, AgentDid, CanonicalMessageEnvelope,
    ContentRetentionClass, DataLayerM0AppendOnlyLedger, DataLayerM0RecordInput,
    DataLayerM0WrappedKey, DataLayerM10ArchiveDueRequest, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10PartitionRecordInput, DataLayerM11ClosureAcceptanceDecision,
    DataLayerM11ClosureEvidenceInput, DataLayerM11OperatorReadinessDecision,
    DataLayerM11OperatorReadinessReport, DataLayerM1MerkleBatch, DataLayerM1MerkleLeaf,
    DataLayerM2AbacEngine, DataLayerM2AccessAuditInput, DataLayerM2AccessAuditLedger,
    DataLayerM2ActorRole, DataLayerM2AuthorizationDecision, DataLayerM2DidAuthRequest,
    DataLayerM2DidSessionService, DataLayerM2MessageScope, DataLayerM3BlindIndexQuery,
    DataLayerM3BlindIndexSearchMode, DataLayerM3MessageMetadataRecord, DataLayerM3SearchCatalog,
    DataLayerM4EscrowDraftInput, DataLayerM4EscrowTransitionAction,
    DataLayerM4EscrowTransitionEngine, DataLayerM5EmbeddingPrivacyMode,
    DataLayerM5EmbeddingRecordInput, DataLayerM5EmbeddingRegistry, DataLayerM6GraphEdgeInput,
    DataLayerM6GraphEdgeRelation, DataLayerM6GraphNodeInput, DataLayerM6GraphNodeKind,
    DataLayerM6GraphRegistry, DataLayerM7BillingQuery, DataLayerM7TelemetryPointInput,
    DataLayerM7TelemetryRegistry, DataLayerM8ComplianceRegistry, DataLayerM8MessageRecordInput,
    DataLayerM8OwnerScopeQuery, DataLayerM8RetentionClass, DataLayerM8WrappedCekInput,
    DataLayerM9DispatchAckStatus, DataLayerM9DispatchRequest, DataLayerM9PresenceConnectRequest,
    DataLayerM9RealtimeDeliveryRegistry, DataLayerPrdCriticalScenarioConformanceDecision,
    DataLayerPrdCriticalScenarioConformanceReport, DirectMessageCiphertext, EnvelopeEncryption,
    EnvelopeHeader, EnvelopeMetadata, EnvelopeProof, ObservabilityHealth, ObservabilitySloProfile,
    CANONICAL_ENCRYPTION_ALGORITHM, CANONICAL_MESSAGE_ENVELOPE_TYPE, CANONICAL_PROOF_PURPOSE,
    DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD, DIRECT_MESSAGE_CIPHER_ALGORITHM,
    DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
use std::collections::BTreeMap;

const SERVICE_API_DATA_LAYER_RUNTIME_EVIDENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.service-api-data-layer-runtime-evidence.v1";
const INITIAL_SERVICE_API_AGENT_REPUTATION_SCORE: u64 = 500;
const INITIAL_SERVICE_API_AGENT_BALANCE: u64 = 100;

mod audit_export;
mod authority_digest;
mod models;
mod persistence;
mod runtime_evidence;
mod store;
mod task_models;
mod task_projection;
#[cfg(test)]
mod tests;

use audit_export::{
    persist_service_api_audit_export_event, resolve_service_api_audit_export_file,
    service_api_message_created_audit_event, service_api_message_relayed_audit_event,
};
use models::*;
use runtime_evidence::build_data_layer_runtime_evidence;
pub(crate) use task_models::ServiceApiSettlementIntentRecord;
use task_models::*;
pub(crate) use task_projection::TaskProjectionError;

pub(crate) use models::{
    ServiceApiAgentBalanceBody, ServiceApiAgentRegistrationStoreError,
    ServiceApiBridgeReceiptRecord, ServiceApiBridgeSettlementTermsRecord, ServiceApiMessageStore,
    ServiceApiPersistedAgentGrantRecord, ServiceApiRelayProgressCounts,
};
pub(crate) use store::escrow_fund_task_id;
#[cfg(test)]
pub(crate) use store::settlement_signature_is_available;
pub(crate) use store::BridgeSettlementIntentInput;
pub(crate) use store::EscrowLifecycleError;
pub(crate) use store::TaskLifecycleError;
pub(crate) use store::{ServiceApiAuthorizationDecision, ServiceApiAuthorizationRequest};

pub(crate) fn bridge_receipt_digest(receipt: &ServiceApiBridgeReceiptRecord) -> String {
    authority_digest::bridge(receipt)
}
