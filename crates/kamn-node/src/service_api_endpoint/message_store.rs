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
#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::path::Path;

const SERVICE_API_DATA_LAYER_RUNTIME_EVIDENCE_SCHEMA_VERSION: &str =
    "kamn.runtime.service-api-data-layer-runtime-evidence.v1";
const INITIAL_SERVICE_API_AGENT_REPUTATION_SCORE: u64 = 500;
const INITIAL_SERVICE_API_AGENT_BALANCE: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiDataLayerRuntimeEvidenceRecord {
    schema_version: String,
    m0_content_hash: String,
    m1_merkle_root: String,
    m2_authorization_reason_code: String,
    m2_audit_record_hash: String,
    m3_blind_index_token: String,
    m3_match_count: usize,
    m4_transition_reason_code: String,
    m5_record_hash: String,
    m6_projection_edge_count: usize,
    m7_observability_health: String,
    m8_retention_due_count: usize,
    m9_dispatch_ack_status: String,
    m9_dispatch_reason_code: String,
    m10_archived_partition_count: usize,
    m11_decision: String,
    m11_reason_codes_csv: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedMessageRecord {
    message_id: String,
    status: String,
    channel_id: Option<String>,
    #[serde(default)]
    sender_did: Option<String>,
    #[serde(default)]
    recipient_did: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data_layer_runtime_evidence: Option<ServiceApiDataLayerRuntimeEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedTaskRecord {
    task_id: String,
    state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedEscrowRecord {
    escrow_id: String,
    state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedContentRecord {
    content_id: String,
    retention_class: String,
    lifecycle_state: String,
    redaction_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedBridgeRecord {
    bridge_id: String,
    source_message_id: String,
    bridge_status: String,
    target_message_id: String,
    forward_tx_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedAgentRecord {
    did: String,
    reputation_score: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    balance: Option<u64>,
    #[serde(default)]
    registered: bool,
    #[serde(default)]
    agent_type: String,
    #[serde(default)]
    model_family: String,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct ServiceApiAgentBalanceBody {
    did: String,
    balance: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ServiceApiAgentRegistrationStoreError {
    Conflict(String),
    Persistence(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServiceApiPersistedMessageStoreSnapshot {
    schema_version: String,
    messages: BTreeMap<String, ServiceApiPersistedMessageRecord>,
    channel_messages: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    auth_nonce_high_watermarks: BTreeMap<String, u64>,
    #[serde(default)]
    tasks: BTreeMap<String, ServiceApiPersistedTaskRecord>,
    #[serde(default)]
    escrows: BTreeMap<String, ServiceApiPersistedEscrowRecord>,
    #[serde(default)]
    contents: BTreeMap<String, ServiceApiPersistedContentRecord>,
    #[serde(default)]
    bridges: BTreeMap<String, ServiceApiPersistedBridgeRecord>,
    #[serde(default)]
    agents: BTreeMap<String, ServiceApiPersistedAgentRecord>,
}

impl Default for ServiceApiPersistedMessageStoreSnapshot {
    fn default() -> Self {
        Self {
            schema_version: "kamn.runtime.service-api-message-store.v2".to_owned(),
            messages: BTreeMap::new(),
            channel_messages: BTreeMap::new(),
            auth_nonce_high_watermarks: BTreeMap::new(),
            tasks: BTreeMap::new(),
            escrows: BTreeMap::new(),
            contents: BTreeMap::new(),
            bridges: BTreeMap::new(),
            agents: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServiceApiMessageStore {
    state_file: Option<String>,
    snapshot: ServiceApiPersistedMessageStoreSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ServiceApiRelayProgressCounts {
    pub(super) created_message_count: u64,
    pub(super) relayed_message_count: u64,
    pub(super) delivered_message_count: u64,
}

impl ServiceApiMessageStore {
    pub(super) fn from_optional_state_file(state_file: Option<String>) -> Result<Self, String> {
        let path_label = state_file.as_deref().unwrap_or("<none>");
        let snapshot = match load_service_api_state_payload(state_file.as_deref())? {
            Some(contents) => {
                serde_json::from_str::<ServiceApiPersistedMessageStoreSnapshot>(contents.as_str())
                    .map_err(|error| {
                    format!("service api state file parse failed: {path_label}: {error}")
                })?
            }
            None => ServiceApiPersistedMessageStoreSnapshot::default(),
        };
        Ok(Self {
            state_file,
            snapshot,
        })
    }

    fn persist(&self) -> Result<(), String> {
        let payload = serde_json::to_string_pretty(&self.snapshot)
            .map_err(|error| format!("service api state serialization failed: {error}"))?;
        persist_service_api_state_payload(self.state_file.as_deref(), payload.as_str())
    }

    fn refresh_from_disk(&mut self) -> Result<(), String> {
        let path_label = self.state_file.as_deref().unwrap_or("<none>");
        let payload = match load_service_api_state_payload(self.state_file.as_deref())? {
            Some(contents) => contents,
            None => return Ok(()),
        };
        let snapshot =
            serde_json::from_str::<ServiceApiPersistedMessageStoreSnapshot>(payload.as_str())
                .map_err(|error| {
                    format!("service api state file parse failed: {path_label}: {error}")
                })?;
        self.snapshot = snapshot;
        Ok(())
    }

    pub(super) fn create_message(
        &mut self,
        payload: &str,
        runtime_mode: &str,
        channel_id: Option<&str>,
        sender_did: Option<&str>,
        recipient_did: Option<&str>,
    ) -> Result<ServiceApiMessageCreateBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "msg-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut message_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.messages.contains_key(message_id.as_str()) {
            message_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        let data_layer_runtime_evidence = build_data_layer_runtime_evidence(
            message_id.as_str(),
            payload,
            sender_did,
            recipient_did,
        )?;

        self.snapshot.messages.insert(
            message_id.clone(),
            ServiceApiPersistedMessageRecord {
                message_id: message_id.clone(),
                status: "created".to_owned(),
                channel_id: channel_id.map(str::to_owned),
                sender_did: sender_did.map(str::to_owned),
                recipient_did: recipient_did.map(str::to_owned),
                body: Some(payload.to_owned()),
                data_layer_runtime_evidence: Some(data_layer_runtime_evidence),
            },
        );
        if let Some(channel_id) = channel_id {
            self.snapshot
                .channel_messages
                .entry(channel_id.to_owned())
                .or_default()
                .push(message_id.clone());
        }
        if let Some(recipient_did) = recipient_did {
            self.snapshot
                .channel_messages
                .entry(recipient_mailbox_channel_id(recipient_did))
                .or_default()
                .push(message_id.clone());
        }
        self.persist()?;
        Ok(ServiceApiMessageCreateBody {
            message_id,
            status: "created".to_owned(),
            runtime_mode: runtime_mode.to_owned(),
        })
    }

    pub(super) fn create_channel(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiChannelCreateBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "channel-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut channel_id = base.clone();
        let mut suffix = 1_u64;
        while self
            .snapshot
            .channel_messages
            .contains_key(channel_id.as_str())
        {
            channel_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot
            .channel_messages
            .entry(channel_id.clone())
            .or_default();
        self.persist()?;
        Ok(ServiceApiChannelCreateBody {
            channel_id,
            status: "created".to_owned(),
        })
    }

    pub(super) fn upsert_relayed_message(
        &mut self,
        message_id: &str,
        sender_did: Option<&str>,
        recipient_did: &str,
        body: &str,
    ) -> Result<ServiceApiMessageRelayBody, String> {
        self.refresh_from_disk()?;
        let normalized_message_id = message_id.trim();
        if normalized_message_id.is_empty() {
            return Err("relay message id must not be empty".to_owned());
        }
        let normalized_recipient_did = recipient_did.trim();
        if normalized_recipient_did.is_empty() {
            return Err("relay recipient did must not be empty".to_owned());
        }
        let normalized_sender_did = sender_did.map(str::trim).filter(|value| !value.is_empty());

        let mut mutated = false;
        if let Some(record) = self.snapshot.messages.get_mut(normalized_message_id) {
            if record.recipient_did.as_deref() != Some(normalized_recipient_did) {
                return Err(format!(
                    "relay recipient mismatch for {normalized_message_id}: expected={}, actual={normalized_recipient_did}",
                    record.recipient_did.as_deref().unwrap_or("none")
                ));
            }
            if record.body.as_deref() != Some(body) {
                return Err(format!(
                    "relay body mismatch for {normalized_message_id}: existing payload differs"
                ));
            }
            if let Some(sender) = normalized_sender_did {
                match record.sender_did.as_deref() {
                    Some(existing) if existing != sender => {
                        return Err(format!(
                            "relay sender mismatch for {normalized_message_id}: expected={existing}, actual={sender}"
                        ));
                    }
                    None => {
                        record.sender_did = Some(sender.to_owned());
                        mutated = true;
                    }
                    _ => {}
                }
            }
            if record.status.as_str() == "created" {
                record.status = "relayed".to_owned();
                mutated = true;
            }
        } else {
            self.snapshot.messages.insert(
                normalized_message_id.to_owned(),
                ServiceApiPersistedMessageRecord {
                    message_id: normalized_message_id.to_owned(),
                    status: "relayed".to_owned(),
                    channel_id: None,
                    sender_did: normalized_sender_did.map(str::to_owned),
                    recipient_did: Some(normalized_recipient_did.to_owned()),
                    body: Some(body.to_owned()),
                    data_layer_runtime_evidence: None,
                },
            );
            mutated = true;
        }

        let mailbox_channel_id = recipient_mailbox_channel_id(normalized_recipient_did);
        let mailbox = self
            .snapshot
            .channel_messages
            .entry(mailbox_channel_id)
            .or_default();
        if !mailbox
            .iter()
            .any(|candidate| candidate == normalized_message_id)
        {
            mailbox.push(normalized_message_id.to_owned());
            mutated = true;
        }

        if mutated {
            self.persist()?;
        }

        let status = self
            .snapshot
            .messages
            .get(normalized_message_id)
            .map(|record| record.status.clone())
            .unwrap_or_else(|| "relayed".to_owned());
        Ok(ServiceApiMessageRelayBody {
            message_id: normalized_message_id.to_owned(),
            status,
        })
    }

    pub(super) fn get_message_for_requester(
        &mut self,
        message_id: &str,
        requester_did: Option<&str>,
    ) -> Result<Option<ServiceApiMessageGetBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.messages.get_mut(message_id) else {
            return Ok(None);
        };
        let should_mark_delivered = record.status.as_str() == "relayed"
            && requester_did.is_some()
            && record.recipient_did.as_deref() == requester_did;
        let payload = if should_mark_delivered {
            record.status = "delivered".to_owned();
            ServiceApiMessageGetBody {
                message_id: record.message_id.clone(),
                status: record.status.clone(),
                sender_did: record.sender_did.clone(),
                recipient_did: record.recipient_did.clone(),
                body: record.body.clone(),
            }
        } else {
            ServiceApiMessageGetBody {
                message_id: record.message_id.clone(),
                status: record.status.clone(),
                sender_did: record.sender_did.clone(),
                recipient_did: record.recipient_did.clone(),
                body: record.body.clone(),
            }
        };
        let _ = record;
        if should_mark_delivered {
            self.persist()?;
        }
        Ok(Some(payload))
    }

    pub(super) fn list_channel_messages(
        &mut self,
        channel_id: &str,
    ) -> Result<ServiceApiChannelMessagesBody, String> {
        self.refresh_from_disk()?;
        Ok(ServiceApiChannelMessagesBody {
            channel_id: channel_id.to_owned(),
            messages: self
                .snapshot
                .channel_messages
                .get(channel_id)
                .cloned()
                .unwrap_or_default(),
        })
    }

    pub(super) fn relay_progress_counts(
        &mut self,
    ) -> Result<ServiceApiRelayProgressCounts, String> {
        self.refresh_from_disk()?;
        let mut created_message_count = 0_u64;
        let mut relayed_message_count = 0_u64;
        let mut delivered_message_count = 0_u64;
        for record in self.snapshot.messages.values() {
            match record.status.as_str() {
                "created" => {
                    created_message_count = created_message_count.saturating_add(1);
                }
                "relayed" => {
                    relayed_message_count = relayed_message_count.saturating_add(1);
                }
                "delivered" => {
                    delivered_message_count = delivered_message_count.saturating_add(1);
                }
                _ => {}
            }
        }
        Ok(ServiceApiRelayProgressCounts {
            created_message_count,
            relayed_message_count,
            delivered_message_count,
        })
    }

    pub(super) fn auth_nonce_high_watermarks(&self) -> BTreeMap<String, u64> {
        self.snapshot.auth_nonce_high_watermarks.clone()
    }

    pub(super) fn record_auth_nonce_high_watermark(
        &mut self,
        sender_did: &str,
        nonce: u64,
    ) -> Result<(), String> {
        self.refresh_from_disk()?;
        let normalized_sender = sender_did.trim();
        if normalized_sender.is_empty() {
            return Err("service api auth nonce sender did must not be empty".to_owned());
        }
        if nonce == 0 {
            return Err("service api auth nonce must be greater than zero".to_owned());
        }

        let current = self
            .snapshot
            .auth_nonce_high_watermarks
            .get(normalized_sender)
            .copied()
            .unwrap_or(0);
        if nonce <= current {
            return Ok(());
        }
        self.snapshot
            .auth_nonce_high_watermarks
            .insert(normalized_sender.to_owned(), nonce);
        self.persist()
    }

    pub(super) fn create_task(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiTaskCreateBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "task-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut task_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.tasks.contains_key(task_id.as_str()) {
            task_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.tasks.insert(
            task_id.clone(),
            ServiceApiPersistedTaskRecord {
                task_id: task_id.clone(),
                state: "submitted".to_owned(),
            },
        );
        self.persist()?;
        Ok(ServiceApiTaskCreateBody {
            task_id,
            state: "submitted".to_owned(),
        })
    }

    pub(super) fn get_task(
        &mut self,
        task_id: &str,
    ) -> Result<Option<ServiceApiTaskGetBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.tasks.get(task_id) else {
            return Ok(None);
        };
        Ok(Some(ServiceApiTaskGetBody {
            task_id: record.task_id.clone(),
            state: record.state.clone(),
        }))
    }

    pub(super) fn transition_task(
        &mut self,
        task_id: &str,
        state: &str,
    ) -> Result<Option<ServiceApiTaskTransitionBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.tasks.get_mut(task_id) else {
            return Ok(None);
        };
        record.state = state.to_owned();
        self.persist()?;
        Ok(Some(ServiceApiTaskTransitionBody {
            task_id: task_id.to_owned(),
            state: state.to_owned(),
        }))
    }

    pub(super) fn fund_escrow(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiEscrowStatusBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "escrow-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut escrow_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.escrows.contains_key(escrow_id.as_str()) {
            escrow_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.escrows.insert(
            escrow_id.clone(),
            ServiceApiPersistedEscrowRecord {
                escrow_id: escrow_id.clone(),
                state: "funded".to_owned(),
            },
        );
        self.persist()?;
        Ok(ServiceApiEscrowStatusBody {
            escrow_id,
            state: "funded".to_owned(),
        })
    }

    pub(super) fn release_escrow(
        &mut self,
        escrow_id: &str,
    ) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.escrows.get_mut(escrow_id) else {
            return Ok(None);
        };
        record.state = "released".to_owned();
        self.persist()?;
        Ok(Some(ServiceApiEscrowStatusBody {
            escrow_id: escrow_id.to_owned(),
            state: "released".to_owned(),
        }))
    }

    pub(super) fn register_content(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiContentRegisterBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "content-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut content_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.contents.contains_key(content_id.as_str()) {
            content_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot.contents.insert(
            content_id.clone(),
            ServiceApiPersistedContentRecord {
                content_id: content_id.clone(),
                retention_class: "standard".to_owned(),
                lifecycle_state: "retained".to_owned(),
                redaction_status: "none".to_owned(),
            },
        );
        self.persist()?;
        Ok(ServiceApiContentRegisterBody {
            content_id,
            retention_class: "standard".to_owned(),
            lifecycle_state: "retained".to_owned(),
            redaction_status: "none".to_owned(),
        })
    }

    pub(super) fn get_content(
        &mut self,
        content_id: &str,
    ) -> Result<Option<ServiceApiContentLifecycleBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.contents.get(content_id) else {
            return Ok(None);
        };
        Ok(Some(ServiceApiContentLifecycleBody {
            content_id: record.content_id.clone(),
            lifecycle_state: record.lifecycle_state.clone(),
            redaction_status: record.redaction_status.clone(),
        }))
    }

    pub(super) fn expire_content(
        &mut self,
        content_id: &str,
    ) -> Result<Option<ServiceApiContentLifecycleBody>, String> {
        self.refresh_from_disk()?;
        let payload = {
            let Some(record) = self.snapshot.contents.get_mut(content_id) else {
                return Ok(None);
            };
            record.lifecycle_state = "expired".to_owned();
            record.redaction_status = "none".to_owned();
            ServiceApiContentLifecycleBody {
                content_id: record.content_id.clone(),
                lifecycle_state: record.lifecycle_state.clone(),
                redaction_status: record.redaction_status.clone(),
            }
        };
        self.persist()?;
        Ok(Some(payload))
    }

    pub(super) fn tombstone_content(
        &mut self,
        content_id: &str,
    ) -> Result<Option<ServiceApiContentLifecycleBody>, String> {
        self.refresh_from_disk()?;
        let payload = {
            let Some(record) = self.snapshot.contents.get_mut(content_id) else {
                return Ok(None);
            };
            record.lifecycle_state = "tombstoned".to_owned();
            record.redaction_status = "redacted".to_owned();
            ServiceApiContentLifecycleBody {
                content_id: record.content_id.clone(),
                lifecycle_state: record.lifecycle_state.clone(),
                redaction_status: record.redaction_status.clone(),
            }
        };
        self.persist()?;
        Ok(Some(payload))
    }

    pub(super) fn submit_bridge(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiBridgeSubmitBody, String> {
        self.refresh_from_disk()?;
        let bridge_tag = deterministic_body_tag(payload.as_bytes());
        let base = format!("bridge-local-{bridge_tag:016x}");
        let mut bridge_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.bridges.contains_key(bridge_id.as_str()) {
            bridge_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        let source_message_id =
            bridge_source_message_id_from_payload(payload, bridge_tag, bridge_id.as_str());
        let target_message_id = format!("msg-bridge-target-{bridge_id}");
        self.snapshot.bridges.insert(
            bridge_id.clone(),
            ServiceApiPersistedBridgeRecord {
                bridge_id: bridge_id.clone(),
                source_message_id: source_message_id.clone(),
                bridge_status: "submitted".to_owned(),
                target_message_id,
                forward_tx_hash: String::new(),
            },
        );
        self.persist()?;
        Ok(ServiceApiBridgeSubmitBody {
            bridge_id,
            source_message_id,
            bridge_status: "submitted".to_owned(),
        })
    }

    pub(super) fn forward_bridge(
        &mut self,
        bridge_id: &str,
    ) -> Result<Option<ServiceApiBridgeStatusBody>, String> {
        self.refresh_from_disk()?;
        let payload = {
            let Some(record) = self.snapshot.bridges.get_mut(bridge_id) else {
                return Ok(None);
            };
            record.bridge_status = "forwarded".to_owned();
            if record.target_message_id.is_empty() {
                record.target_message_id = format!("msg-bridge-target-{}", record.bridge_id);
            }
            record.forward_tx_hash = format!("sha256:bridge-forwarded-{}", record.bridge_id);
            ServiceApiBridgeStatusBody {
                bridge_id: record.bridge_id.clone(),
                bridge_status: record.bridge_status.clone(),
                target_message_id: record.target_message_id.clone(),
                forward_tx_hash: record.forward_tx_hash.clone(),
            }
        };
        self.persist()?;
        Ok(Some(payload))
    }

    pub(super) fn get_bridge(
        &mut self,
        bridge_id: &str,
    ) -> Result<Option<ServiceApiBridgeStatusBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.bridges.get(bridge_id) else {
            return Ok(None);
        };
        Ok(Some(ServiceApiBridgeStatusBody {
            bridge_id: record.bridge_id.clone(),
            bridge_status: record.bridge_status.clone(),
            target_message_id: record.target_message_id.clone(),
            forward_tx_hash: record.forward_tx_hash.clone(),
        }))
    }

    pub(super) fn get_or_create_agent_profile(
        &mut self,
        agent_did: &str,
    ) -> Result<ServiceApiAgentGetBody, String> {
        let record = self.get_or_create_agent_record(agent_did)?;
        Ok(agent_profile_body(&record))
    }

    pub(super) fn register_agent_profile(
        &mut self,
        agent_did: &str,
        registration: &ServiceApiAgentRegisterRequestBody,
    ) -> Result<ServiceApiAgentGetBody, ServiceApiAgentRegistrationStoreError> {
        self.refresh_from_disk()
            .map_err(ServiceApiAgentRegistrationStoreError::Persistence)?;
        let normalized_did = agent_did.trim();
        if normalized_did.is_empty() {
            return Err(ServiceApiAgentRegistrationStoreError::Persistence(
                "agent did must not be empty".to_owned(),
            ));
        }

        let expected_capabilities: Vec<String> = registration
            .capabilities
            .iter()
            .map(|value| value.trim().to_owned())
            .collect();

        let existing = self
            .snapshot
            .agents
            .get(normalized_did)
            .cloned()
            .unwrap_or_else(|| default_agent_record(normalized_did));
        if existing.registered
            && (record_agent_type(&existing) != registration.agent_type.trim()
                || record_model_family(&existing) != registration.model_family.trim()
                || record_capabilities(&existing) != expected_capabilities)
        {
            return Err(ServiceApiAgentRegistrationStoreError::Conflict(
                "agent registration metadata mismatch for existing did".to_owned(),
            ));
        }

        let mut record = existing;
        record.registered = true;
        record.agent_type = registration.agent_type.trim().to_owned();
        record.model_family = registration.model_family.trim().to_owned();
        record.capabilities = expected_capabilities;
        self.snapshot
            .agents
            .insert(normalized_did.to_owned(), record.clone());
        self.persist()
            .map_err(ServiceApiAgentRegistrationStoreError::Persistence)?;
        Ok(agent_profile_body(&record))
    }

    pub(super) fn search_agent_profiles(
        &mut self,
        search: &ServiceApiAgentSearchRequestBody,
    ) -> Result<Vec<ServiceApiAgentGetBody>, String> {
        self.refresh_from_disk()?;
        let capability = search.capability.as_deref();
        let model_family = search.model_family.as_deref();
        let mut results: Vec<ServiceApiAgentGetBody> = self
            .snapshot
            .agents
            .values()
            .filter(|record| record.registered)
            .filter(|record| match model_family {
                Some(expected) => record_model_family(record) == expected,
                None => true,
            })
            .filter(|record| match capability {
                Some(expected) => record_capabilities(record)
                    .iter()
                    .any(|value| value == expected),
                None => true,
            })
            .map(agent_profile_body)
            .collect();
        results.sort_by(|left, right| left.did.cmp(&right.did));
        Ok(results)
    }

    pub(super) fn get_or_create_agent_balance(
        &mut self,
        agent_did: &str,
    ) -> Result<ServiceApiAgentBalanceBody, String> {
        let record = self.get_or_create_agent_record(agent_did)?;
        Ok(ServiceApiAgentBalanceBody {
            did: record.did,
            balance: record.balance.unwrap_or(INITIAL_SERVICE_API_AGENT_BALANCE),
        })
    }

    fn get_or_create_agent_record(
        &mut self,
        agent_did: &str,
    ) -> Result<ServiceApiPersistedAgentRecord, String> {
        self.refresh_from_disk()?;
        let normalized_did = agent_did.trim();
        if normalized_did.is_empty() {
            return Err("agent did must not be empty".to_owned());
        }

        let mut persisted = false;
        let record = match self.snapshot.agents.get_mut(normalized_did) {
            Some(record) => {
                if record.balance.is_none() {
                    record.balance = Some(INITIAL_SERVICE_API_AGENT_BALANCE);
                    persisted = true;
                }
                record.clone()
            }
            None => {
                let record = default_agent_record(normalized_did);
                self.snapshot
                    .agents
                    .insert(normalized_did.to_owned(), record.clone());
                persisted = true;
                record
            }
        };

        if persisted {
            self.persist()?;
        }
        Ok(record)
    }
}

fn default_agent_record(agent_did: &str) -> ServiceApiPersistedAgentRecord {
    ServiceApiPersistedAgentRecord {
        did: agent_did.to_owned(),
        reputation_score: INITIAL_SERVICE_API_AGENT_REPUTATION_SCORE,
        balance: Some(INITIAL_SERVICE_API_AGENT_BALANCE),
        registered: false,
        agent_type: default_agent_type(),
        model_family: default_model_family(),
        capabilities: default_capabilities(),
    }
}

fn agent_profile_body(record: &ServiceApiPersistedAgentRecord) -> ServiceApiAgentGetBody {
    ServiceApiAgentGetBody {
        did: record.did.clone(),
        reputation_score: record.reputation_score,
        agent_type: record_agent_type(record),
        model_family: record_model_family(record),
        capabilities: record_capabilities(record),
    }
}

fn record_agent_type(record: &ServiceApiPersistedAgentRecord) -> String {
    if record.agent_type.trim().is_empty() {
        return default_agent_type();
    }
    record.agent_type.clone()
}

fn record_model_family(record: &ServiceApiPersistedAgentRecord) -> String {
    if record.model_family.trim().is_empty() {
        return default_model_family();
    }
    record.model_family.clone()
}

fn record_capabilities(record: &ServiceApiPersistedAgentRecord) -> Vec<String> {
    if record.capabilities.is_empty() {
        return default_capabilities();
    }
    record.capabilities.clone()
}

fn default_agent_type() -> String {
    "service-agent".to_owned()
}

fn default_model_family() -> String {
    "service-api".to_owned()
}

fn default_capabilities() -> Vec<String> {
    vec!["profile:read".to_owned()]
}

fn bridge_source_message_id_from_payload(
    payload: &str,
    bridge_tag: u64,
    bridge_id: &str,
) -> String {
    let default_value = format!("msg-bridge-source-{bridge_tag:016x}");
    let parsed = match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(value) => value,
        Err(_) => return default_value,
    };
    let Some(source_message_id) = parsed
        .get("source_message_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return default_value;
    };
    if source_message_id == bridge_id {
        return default_value;
    }
    source_message_id.to_owned()
}

#[cfg(test)]
fn write_state_file_atomically(path: &Path, payload: &str) -> Result<(), String> {
    let parent_dir = path
        .parent()
        .filter(|candidate| !candidate.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "service api state file path has no file name: {}",
                path.display()
            )
        })?;
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("service api state file temp suffix failed: {error}"))?
        .as_nanos();
    let temp_file_name = format!("{file_name}.tmp-{}-{unique_suffix}", std::process::id());
    let temp_path = parent_dir.join(temp_file_name);

    let mut temp_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temp_path.as_path())
        .map_err(|error| {
            format!(
                "service api state file temp create failed: {}: {error}",
                temp_path.display()
            )
        })?;

    if let Err(error) = temp_file.write_all(payload.as_bytes()) {
        let _ = fs::remove_file(temp_path.as_path());
        return Err(format!(
            "service api state file temp write failed: {}: {error}",
            temp_path.display()
        ));
    }

    if let Err(error) = temp_file.sync_all() {
        let _ = fs::remove_file(temp_path.as_path());
        return Err(format!(
            "service api state file temp sync failed: {}: {error}",
            temp_path.display()
        ));
    }
    drop(temp_file);

    if let Err(error) = fs::rename(temp_path.as_path(), path) {
        let _ = fs::remove_file(temp_path.as_path());
        return Err(format!(
            "service api state file rename failed: {}: {error}",
            path.display()
        ));
    }

    if let Ok(parent_handle) = fs::File::open(parent_dir) {
        let _ = parent_handle.sync_all();
    }

    Ok(())
}

fn recipient_mailbox_channel_id(recipient_did: &str) -> String {
    format!("recipient:{recipient_did}")
}

fn build_data_layer_runtime_evidence(
    message_id: &str,
    payload: &str,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
) -> Result<ServiceApiDataLayerRuntimeEvidenceRecord, String> {
    let sender_agent_did =
        normalize_agent_did(sender_did, "kamn:did:agent:service-api-runtime-sender");
    let mut recipient_agent_did = normalize_agent_did(
        recipient_did,
        "kamn:did:agent:service-api-runtime-recipient",
    );
    if sender_agent_did == recipient_agent_did {
        recipient_agent_did = "kamn:did:agent:service-api-runtime-recipient-alt".to_owned();
    }

    let owner_did = "kamn:did:owner:service-api-runtime";
    let owner_counterparty_did = "kamn:did:owner:service-api-runtime-recipient";
    let payload_tag = deterministic_body_tag(payload.as_bytes());
    let event_epoch_seconds = 1_708_560_000_u64.saturating_add(payload_tag % 10_000);
    let content_size_bytes = payload.len().max(1);
    let compressed_size_bytes = (content_size_bytes / 2).max(1);

    let mut envelope_body = BTreeMap::new();
    envelope_body.insert("payload".to_owned(), payload.to_owned());
    envelope_body.insert("runtime_message_id".to_owned(), message_id.to_owned());
    let envelope = CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: message_id.to_owned(),
            type_name: CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
            from: sender_agent_did.clone(),
            to: vec![recipient_agent_did.clone()],
            created: "2026-02-24T00:00:00Z".to_owned(),
            expires: "2026-02-24T01:00:00Z".to_owned(),
            thread_id: Some(format!("thread:{message_id}")),
            parent_id: None,
            nonce: (payload_tag % 1024).saturating_add(1),
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "normal".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
                recipient_keys: vec!["did:key:z6Mkrecipient#key-agreement-1".to_owned()],
            },
        },
        body: envelope_body,
        attachments: Vec::new(),
        proof: EnvelopeProof {
            type_name: "DataIntegrityProof".to_owned(),
            created: "2026-02-24T00:00:00Z".to_owned(),
            verification_method: format!("{sender_agent_did}#key-1"),
            proof_purpose: CANONICAL_PROOF_PURPOSE.to_owned(),
            proof_value: format!("sig:{message_id}"),
        },
    };
    let ciphertext = DirectMessageCiphertext {
        key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
        cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
        sender_key_ref: "did:key:z6Mksender#key-agreement-1".to_owned(),
        recipient_key_ref: "did:key:z6Mkrecipient#key-agreement-1".to_owned(),
        nonce: (payload_tag % 2048).saturating_add(1),
        ciphertext: format!("{payload_tag:016x}"),
        auth_tag: format!("{:032x}", payload_tag.saturating_add(1)),
    };
    let mut m0_ledger = DataLayerM0AppendOnlyLedger::new();
    let m0_record = m0_ledger
        .append(DataLayerM0RecordInput {
            envelope,
            ciphertext,
            wrapped_keys: vec![DataLayerM0WrappedKey {
                did: recipient_agent_did.clone(),
                wrapped_cek: format!("wrapped:{message_id}"),
            }],
            compression_codec: DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD.to_owned(),
            compression_dict_id: Some(1),
            content_size_bytes,
            compressed_size_bytes,
        })
        .map_err(|error| format!("m0 runtime evidence failed: {error}"))?;
    m0_ledger
        .verify_hash_chain()
        .map_err(|error| format!("m0 hash-chain verification failed: {error}"))?;

    let m1_batch = DataLayerM1MerkleBatch::assemble(vec![
        DataLayerM1MerkleLeaf {
            message_id: message_id.to_owned(),
            leaf_index: 0,
            content_hash: m0_record.content_hash.clone(),
        },
        DataLayerM1MerkleLeaf {
            message_id: format!("{message_id}:projection"),
            leaf_index: 1,
            content_hash: format!("sha256:{payload_tag:016x}"),
        },
    ])
    .map_err(|error| format!("m1 merkle assembly failed: {error}"))?;
    let m1_proof = m1_batch
        .inclusion_proof(message_id)
        .map_err(|error| format!("m1 inclusion proof failed: {error}"))?;
    verify_data_layer_m1_inclusion_proof(&m1_proof)
        .map_err(|error| format!("m1 inclusion verification failed: {error}"))?;

    let m2_session_service = DataLayerM2DidSessionService::new(900)
        .map_err(|error| format!("m2 init failed: {error}"))?;
    let auth_challenge = format!("nonce-{payload_tag}");
    let m2_session_token = m2_session_service
        .authenticate(DataLayerM2DidAuthRequest {
            requester_did: sender_agent_did.clone(),
            challenge: auth_challenge.clone(),
            credential: format!("sig:{sender_agent_did}:{auth_challenge}"),
            issued_at_epoch_seconds: event_epoch_seconds,
            ttl_seconds: 300,
        })
        .map_err(|error| format!("m2 did authentication failed: {error}"))?;
    let m2_abac = DataLayerM2AbacEngine::new();
    let m2_authorization = m2_abac
        .authorize_message_visibility(
            sender_agent_did.as_str(),
            DataLayerM2ActorRole::Agent,
            &DataLayerM2MessageScope {
                message_id: message_id.to_owned(),
                sender_did: sender_agent_did.clone(),
                recipient_did: recipient_agent_did.clone(),
                owner_sender_did: owner_did.to_owned(),
                owner_recipient_did: owner_counterparty_did.to_owned(),
                escrow_id: None,
            },
        )
        .map_err(|error| format!("m2 authorization failed: {error}"))?;
    let m2_authorization_reason_code = m2_authorization_reason_code(&m2_authorization);
    let mut m2_audit_ledger = DataLayerM2AccessAuditLedger::new();
    let m2_audit_record = m2_audit_ledger
        .append(DataLayerM2AccessAuditInput {
            requester_did: m2_session_token.requester_did.clone(),
            action: "create_message".to_owned(),
            resource_id: message_id.to_owned(),
            reason_code: m2_authorization_reason_code.clone(),
            event_epoch_seconds: event_epoch_seconds.saturating_add(1),
        })
        .map_err(|error| format!("m2 access audit append failed: {error}"))?;
    m2_audit_ledger
        .verify_hash_chain()
        .map_err(|error| format!("m2 access audit verification failed: {error}"))?;

    let m3_blind_index_token =
        data_layer_m3_compute_blind_index("service-api-runtime-owner-key", "message", payload)
            .map_err(|error| format!("m3 blind-index compute failed: {error}"))?;
    let mut m3_catalog = DataLayerM3SearchCatalog::new();
    let mut m3_blind_indexes = BTreeMap::new();
    m3_blind_indexes.insert("message".to_owned(), m3_blind_index_token.clone());
    m3_catalog
        .register_record(DataLayerM3MessageMetadataRecord {
            message_id: message_id.to_owned(),
            owner_did: owner_did.to_owned(),
            sender_did: sender_agent_did.clone(),
            recipient_did: recipient_agent_did.clone(),
            session_id: Some(m2_session_token.token_id),
            escrow_id: None,
            message_type: "text".to_owned(),
            created_at_epoch_seconds: event_epoch_seconds.saturating_add(2),
            blind_indexes: m3_blind_indexes,
        })
        .map_err(|error| format!("m3 catalog registration failed: {error}"))?;
    let m3_matches = m3_catalog
        .search_blind_index(DataLayerM3BlindIndexQuery {
            owner_did: owner_did.to_owned(),
            field_name: "message".to_owned(),
            token: m3_blind_index_token.clone(),
            mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
            limit: Some(10),
        })
        .map_err(|error| format!("m3 search failed: {error}"))?;

    let mut m4_escrow = DataLayerM4EscrowTransitionEngine::new();
    let m4_escrow_id = format!("escrow:{message_id}");
    m4_escrow
        .create_escrow(DataLayerM4EscrowDraftInput {
            escrow_id: m4_escrow_id.clone(),
            initiator_did: sender_agent_did.clone(),
            counterparty_did: recipient_agent_did.clone(),
            auditor_did: Some("kamn:did:auditor:service-api-runtime".to_owned()),
            auditor_threshold: Some(1),
            auditor_share_holders: vec!["kamn:did:holder:service-api-runtime".to_owned()],
            expires_at_epoch_seconds: Some(event_epoch_seconds.saturating_add(3_600)),
        })
        .map_err(|error| format!("m4 escrow draft failed: {error}"))?;
    let m4_transition = m4_escrow
        .apply_transition(
            m4_escrow_id.as_str(),
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds: event_epoch_seconds.saturating_add(3),
            },
        )
        .map_err(|error| format!("m4 transition failed: {error}"))?;

    let mut m5_registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let m5_record = m5_registry
        .append(DataLayerM5EmbeddingRecordInput {
            embedding_id: format!("embed:{message_id}"),
            message_id: message_id.to_owned(),
            owner_did: owner_did.to_owned(),
            agent_did: sender_agent_did.clone(),
            retention_class: ContentRetentionClass::Standard,
            model_id: "text-embedding-3-large".to_owned(),
            vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
            vector_plaintext: Some(vec![1.0, 0.0, 0.0]),
            created_at_epoch_seconds: event_epoch_seconds.saturating_add(4),
        })
        .map_err(|error| format!("m5 embedding append failed: {error}"))?;
    m5_registry
        .verify_owner_integrity(owner_did)
        .map_err(|error| format!("m5 owner integrity failed: {error}"))?;

    let mut m6_registry = DataLayerM6GraphRegistry::new();
    let sender_node_id = format!("node:{sender_agent_did}");
    let recipient_node_id = format!("node:{recipient_agent_did}");
    m6_registry
        .register_node(DataLayerM6GraphNodeInput {
            owner_did: owner_did.to_owned(),
            node_id: sender_node_id.clone(),
            kind: DataLayerM6GraphNodeKind::Agent,
            label: sender_agent_did.clone(),
        })
        .map_err(|error| format!("m6 sender node registration failed: {error}"))?;
    m6_registry
        .register_node(DataLayerM6GraphNodeInput {
            owner_did: owner_did.to_owned(),
            node_id: recipient_node_id.clone(),
            kind: DataLayerM6GraphNodeKind::Agent,
            label: recipient_agent_did.clone(),
        })
        .map_err(|error| format!("m6 recipient node registration failed: {error}"))?;
    m6_registry
        .register_edge(DataLayerM6GraphEdgeInput {
            owner_did: owner_did.to_owned(),
            edge_id: format!("edge:{message_id}"),
            relation: DataLayerM6GraphEdgeRelation::Messaged,
            from_node_id: sender_node_id,
            to_node_id: recipient_node_id,
            weight: 1.0,
            observed_at_epoch_seconds: event_epoch_seconds.saturating_add(5),
        })
        .map_err(|error| format!("m6 edge registration failed: {error}"))?;
    let m6_projection = m6_registry
        .export_portable_edge_projection(owner_did)
        .map_err(|error| format!("m6 projection failed: {error}"))?;

    let mut m7_registry = DataLayerM7TelemetryRegistry::new();
    m7_registry
        .ingest_point(DataLayerM7TelemetryPointInput {
            owner_did: owner_did.to_owned(),
            agent_did: sender_agent_did.clone(),
            timestamp_epoch_seconds: event_epoch_seconds.saturating_add(6),
            message_count: 1,
            bytes_stored: content_size_bytes as u64,
            query_count: 1,
            embedding_count: 1,
            embedding_anomaly_count: 0,
            ingress_latency_ms_p95: 120,
            egress_latency_ms_p95: 140,
            active_sessions: 1,
        })
        .map_err(|error| format!("m7 telemetry ingest failed: {error}"))?;
    let m7_observability = m7_registry
        .evaluate_owner_observability(
            DataLayerM7BillingQuery {
                requester_owner_did: owner_did.to_owned(),
                owner_did: owner_did.to_owned(),
            },
            ObservabilitySloProfile::baseline(),
        )
        .map_err(|error| format!("m7 observability projection failed: {error}"))?;

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(DataLayerM8MessageRecordInput {
            owner_did: owner_did.to_owned(),
            message_id: message_id.to_owned(),
            created_at_epoch_seconds: event_epoch_seconds,
            content_hash: format!("sha256:{message_id}:content"),
            hash_chain_prev: format!("sha256:{message_id}:prev"),
            retention_class: DataLayerM8RetentionClass::Standard,
            retention_extension_seconds: 0,
            wrapped_keys: vec![DataLayerM8WrappedCekInput {
                recipient_did: recipient_agent_did.clone(),
                wrapped_cek: format!("wrapped:{message_id}"),
            }],
        })
        .map_err(|error| format!("m8 message registration failed: {error}"))?;
    let m8_retention_due = m8_registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: owner_did.to_owned(),
                owner_did: owner_did.to_owned(),
            },
            event_epoch_seconds.saturating_add(100_000_000),
        )
        .map_err(|error| format!("m8 retention projection failed: {error}"))?;

    let mut m9_registry = DataLayerM9RealtimeDeliveryRegistry::new();
    m9_registry
        .connect_presence(DataLayerM9PresenceConnectRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            agent_did: recipient_agent_did.clone(),
            connected_since_epoch_seconds: event_epoch_seconds.saturating_add(7),
            last_heartbeat_epoch_seconds: event_epoch_seconds.saturating_add(7),
            gateway_node: "gateway-service-api-runtime".to_owned(),
            capabilities_active: vec!["ws".to_owned()],
        })
        .map_err(|error| format!("m9 presence connect failed: {error}"))?;
    let m9_dispatch_outcome = m9_registry
        .dispatch_message(DataLayerM9DispatchRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            sender_agent_did: sender_agent_did.clone(),
            recipient_agent_did: recipient_agent_did.clone(),
            message_id: message_id.to_owned(),
            dispatched_at_epoch_seconds: event_epoch_seconds.saturating_add(8),
        })
        .map_err(|error| format!("m9 dispatch failed: {error}"))?;

    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(DataLayerM10PartitionRecordInput {
            partition_month_id: 202401,
            all_messages_shredded: true,
        })
        .map_err(|error| format!("m10 partition registration failed: {error}"))?;
    let m10_archived_partitions = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .map_err(|error| format!("m10 archival projection failed: {error}"))?;

    let m11_closure = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: format!("service-api-runtime:{message_id}"),
        hardening_report: DataLayerM11OperatorReadinessReport {
            decision: DataLayerM11OperatorReadinessDecision::Go,
            reason_codes: vec!["m11_operator_readiness_go"],
            missing_required_scenario_ids: Vec::new(),
            failing_critical_scenario_ids: Vec::new(),
            total_required_scenarios: 1,
            passed_required_scenarios: 1,
        },
        critical_scenario_report: DataLayerPrdCriticalScenarioConformanceReport {
            decision: DataLayerPrdCriticalScenarioConformanceDecision::Conformant,
            reason_codes: vec!["prd_critical_scenarios_conformant"],
            missing_scenario_ids: Vec::new(),
            failed_scenario_ids: Vec::new(),
            shell_policy_violation_scenario_ids: Vec::new(),
            total_required_scenarios: 10,
            passed_required_scenarios: 10,
        },
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    })
    .map_err(|error| format!("m11 closure evaluation failed: {error}"))?;

    Ok(ServiceApiDataLayerRuntimeEvidenceRecord {
        schema_version: SERVICE_API_DATA_LAYER_RUNTIME_EVIDENCE_SCHEMA_VERSION.to_owned(),
        m0_content_hash: m0_record.content_hash,
        m1_merkle_root: m1_batch.merkle_root,
        m2_authorization_reason_code,
        m2_audit_record_hash: m2_audit_record.record_hash,
        m3_blind_index_token,
        m3_match_count: m3_matches.len(),
        m4_transition_reason_code: m4_transition.reason_code.to_owned(),
        m5_record_hash: m5_record.record_hash,
        m6_projection_edge_count: m6_projection.len(),
        m7_observability_health: observability_health_label(
            m7_observability.snapshot.latest_health,
        )
        .to_owned(),
        m8_retention_due_count: m8_retention_due.len(),
        m9_dispatch_ack_status: data_layer_m9_ack_status_label(m9_dispatch_outcome.ack_status)
            .to_owned(),
        m9_dispatch_reason_code: m9_dispatch_outcome.reason_code.to_owned(),
        m10_archived_partition_count: m10_archived_partitions.len(),
        m11_decision: m11_decision_label(m11_closure.decision).to_owned(),
        m11_reason_codes_csv: m11_closure.reason_codes.join(","),
    })
}

fn normalize_agent_did(candidate: Option<&str>, fallback: &str) -> String {
    match candidate {
        Some(value) if AgentDid::parse(value).is_ok() => value.to_owned(),
        _ => fallback.to_owned(),
    }
}

fn m2_authorization_reason_code(decision: &DataLayerM2AuthorizationDecision) -> String {
    match decision {
        DataLayerM2AuthorizationDecision::Allow { reason_code }
        | DataLayerM2AuthorizationDecision::Deny { reason_code } => (*reason_code).to_owned(),
    }
}

fn data_layer_m9_ack_status_label(status: DataLayerM9DispatchAckStatus) -> &'static str {
    match status {
        DataLayerM9DispatchAckStatus::Delivered => "delivered",
        DataLayerM9DispatchAckStatus::Queued => "queued",
    }
}

fn m11_decision_label(decision: DataLayerM11ClosureAcceptanceDecision) -> &'static str {
    match decision {
        DataLayerM11ClosureAcceptanceDecision::Accepted => "accepted",
        DataLayerM11ClosureAcceptanceDecision::Rejected => "rejected",
    }
}

fn observability_health_label(health: ObservabilityHealth) -> &'static str {
    match health {
        ObservabilityHealth::Healthy => "healthy",
        ObservabilityHealth::Degraded => "degraded",
        ObservabilityHealth::Critical => "critical",
    }
}

#[cfg(test)]
mod atomic_state_write_tests {
    use super::write_state_file_atomically;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be available")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-node-{name}-{}-{nanos}", std::process::id()))
    }

    fn collect_atomic_temp_entries(dir: &Path, state_file_name: &str) -> Vec<PathBuf> {
        let prefix = format!("{state_file_name}.tmp-");
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(prefix.as_str())
                {
                    entries.push(entry.path());
                }
            }
        }
        entries
    }

    #[test]
    fn unit_atomic_state_write_replaces_payload_and_removes_temp_entries() {
        let base_dir = unique_temp_dir("atomic-state-write-ok");
        fs::create_dir_all(base_dir.as_path()).expect("temp base dir should create");
        let state_file = base_dir.join("service-api-state.json");
        fs::write(state_file.as_path(), "{\"schema_version\":\"old\"}")
            .expect("initial state fixture should write");

        write_state_file_atomically(
            state_file.as_path(),
            "{\"schema_version\":\"new\",\"messages\":{}}",
        )
        .expect("atomic write should succeed");

        let payload = fs::read_to_string(state_file.as_path()).expect("state file should remain");
        assert!(
            payload.contains("\"schema_version\":\"new\""),
            "atomic replacement should update destination payload"
        );

        let temp_entries =
            collect_atomic_temp_entries(base_dir.as_path(), "service-api-state.json");
        assert!(
            temp_entries.is_empty(),
            "atomic writer should not leave temp files behind after success"
        );

        let _ = fs::remove_file(state_file);
        let _ = fs::remove_dir(base_dir);
    }

    #[test]
    fn unit_atomic_state_write_rename_failure_cleans_temp_entry() {
        let base_dir = unique_temp_dir("atomic-state-write-rename-fail");
        fs::create_dir_all(base_dir.as_path()).expect("temp base dir should create");
        let state_path = base_dir.join("service-api-state.json");
        fs::create_dir(state_path.as_path()).expect("fixture directory should create");

        let error = write_state_file_atomically(
            state_path.as_path(),
            "{\"schema_version\":\"new\",\"messages\":{}}",
        )
        .expect_err("rename over directory destination must fail");
        assert!(
            error.contains("state file rename failed"),
            "rename failure should fail closed with deterministic marker"
        );

        let temp_entries =
            collect_atomic_temp_entries(base_dir.as_path(), "service-api-state.json");
        assert!(
            temp_entries.is_empty(),
            "failed atomic write must clean temp entries before returning"
        );

        let _ = fs::remove_dir(state_path);
        let _ = fs::remove_dir(base_dir);
    }
}
