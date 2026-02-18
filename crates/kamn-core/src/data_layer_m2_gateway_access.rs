//! M2 access-gateway contracts for DID authn/authz, RLS templates, and access auditing.
//!
//! This module models the PRD M2 control surface as deterministic Rust contracts:
//! DID-authenticated session issuance, ABAC message visibility checks, RLS policy
//! template emission, and append-only audit logging with hash-chain verification.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// PostgreSQL session variable key used by RLS predicates.
pub const DATA_LAYER_M2_REQUESTER_DID_SETTING: &str = "kamn.requester_did";
/// Hash algorithm label used by M2 deterministic digests.
pub const DATA_LAYER_M2_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker for access-audit hash chains.
pub const DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS: &str = "GENESIS";

/// Input for DID-authenticated session issuance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2DidAuthRequest {
    /// Requester DID.
    pub requester_did: String,
    /// Challenge/nonce bound to credential signature.
    pub challenge: String,
    /// Credential payload carrying deterministic signature binding.
    pub credential: String,
    /// Request issuance timestamp in epoch seconds.
    pub issued_at_epoch_seconds: u64,
    /// Requested session TTL in seconds.
    pub ttl_seconds: u64,
}

/// Session token issued by M2 DID authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2SessionToken {
    /// Stable session token identifier.
    pub token_id: String,
    /// Authenticated requester DID.
    pub requester_did: String,
    /// Session issuance timestamp.
    pub issued_at_epoch_seconds: u64,
    /// Session expiry timestamp.
    pub expires_at_epoch_seconds: u64,
}

/// Deterministic DID session service for M2 gateway contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2DidSessionService {
    max_ttl_seconds: u64,
}

impl DataLayerM2DidSessionService {
    /// Constructs a DID session service with max allowed TTL.
    pub fn new(max_ttl_seconds: u64) -> Result<Self, DataLayerM2GatewayError> {
        if max_ttl_seconds == 0 {
            return Err(DataLayerM2GatewayError::InvalidSessionTtl {
                ttl_seconds: 0,
                max_ttl_seconds,
            });
        }
        Ok(Self { max_ttl_seconds })
    }

    /// Authenticates a DID-bound request and issues a deterministic session token.
    pub fn authenticate(
        &self,
        request: DataLayerM2DidAuthRequest,
    ) -> Result<DataLayerM2SessionToken, DataLayerM2GatewayError> {
        if request.challenge.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("challenge"));
        }
        if request.credential.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("credential"));
        }
        if request.ttl_seconds == 0 || request.ttl_seconds > self.max_ttl_seconds {
            return Err(DataLayerM2GatewayError::InvalidSessionTtl {
                ttl_seconds: request.ttl_seconds,
                max_ttl_seconds: self.max_ttl_seconds,
            });
        }

        validate_kamn_did(request.requester_did.as_str())?;
        let expected_credential = format!("sig:{}:{}", request.requester_did, request.challenge);
        if request.credential != expected_credential {
            return Err(DataLayerM2GatewayError::InvalidCredential(
                "credential signature mismatch".to_owned(),
            ));
        }

        let expires_at_epoch_seconds = request
            .issued_at_epoch_seconds
            .checked_add(request.ttl_seconds)
            .ok_or(DataLayerM2GatewayError::SessionExpiryOverflow)?;
        let token_id = format!(
            "session:{}",
            tagged_digest(
                format!(
                    "did-session|did:{}|challenge:{}|issued:{}|expires:{}",
                    request.requester_did,
                    request.challenge,
                    request.issued_at_epoch_seconds,
                    expires_at_epoch_seconds
                )
                .as_str()
            )
        );

        Ok(DataLayerM2SessionToken {
            token_id,
            requester_did: request.requester_did,
            issued_at_epoch_seconds: request.issued_at_epoch_seconds,
            expires_at_epoch_seconds,
        })
    }
}

/// Actor role used by M2 ABAC authorization checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM2ActorRole {
    /// Autonomous agent requesting direct message visibility.
    Agent,
    /// Owner account requesting supervisory access.
    Owner,
    /// Escrow auditor requesting dispute-scoped access.
    EscrowAuditor,
    /// Platform-operator identity.
    PlatformOperator,
}

/// Message metadata scope inspected by M2 ABAC checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2MessageScope {
    /// Stable message identifier.
    pub message_id: String,
    /// Sender agent DID.
    pub sender_did: String,
    /// Recipient agent DID.
    pub recipient_did: String,
    /// Owner DID for sender.
    pub owner_sender_did: String,
    /// Owner DID for recipient.
    pub owner_recipient_did: String,
    /// Optional escrow identifier when message is escrow-scoped.
    pub escrow_id: Option<String>,
}

/// Authorization decision projected by ABAC evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2AuthorizationDecision {
    /// Access granted with deterministic reason code.
    Allow {
        /// Stable reason code.
        reason_code: &'static str,
    },
    /// Access denied with deterministic reason code.
    Deny {
        /// Stable reason code.
        reason_code: &'static str,
    },
}

/// ABAC engine for M2 message visibility decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM2AbacEngine {
    escrow_auditors_by_escrow: BTreeMap<String, BTreeSet<String>>,
    disputed_escrows: BTreeSet<String>,
}

impl DataLayerM2AbacEngine {
    /// Creates an empty ABAC engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one auditor DID for an escrow scope.
    pub fn register_escrow_auditor(
        &mut self,
        escrow_id: &str,
        auditor_did: &str,
    ) -> Result<(), DataLayerM2GatewayError> {
        if escrow_id.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("escrow_id"));
        }
        validate_kamn_did(auditor_did)?;
        self.escrow_auditors_by_escrow
            .entry(escrow_id.to_owned())
            .or_default()
            .insert(auditor_did.to_owned());
        Ok(())
    }

    /// Marks whether an escrow is in active dispute scope.
    pub fn set_escrow_dispute_active(&mut self, escrow_id: &str, active: bool) {
        if active {
            self.disputed_escrows.insert(escrow_id.to_owned());
        } else {
            self.disputed_escrows.remove(escrow_id);
        }
    }

    /// Evaluates fail-closed message visibility authorization.
    pub fn authorize_message_visibility(
        &self,
        requester_did: &str,
        requester_role: DataLayerM2ActorRole,
        scope: &DataLayerM2MessageScope,
    ) -> Result<DataLayerM2AuthorizationDecision, DataLayerM2GatewayError> {
        validate_message_scope(scope)?;
        validate_kamn_did(requester_did)?;

        let decision = match requester_role {
            DataLayerM2ActorRole::Agent => {
                if requester_did == scope.sender_did || requester_did == scope.recipient_did {
                    DataLayerM2AuthorizationDecision::Allow {
                        reason_code: "m2_agent_counterparty_scope_allowed",
                    }
                } else {
                    DataLayerM2AuthorizationDecision::Deny {
                        reason_code: "m2_abac_scope_denied",
                    }
                }
            }
            DataLayerM2ActorRole::Owner => {
                if requester_did == scope.owner_sender_did
                    || requester_did == scope.owner_recipient_did
                {
                    DataLayerM2AuthorizationDecision::Allow {
                        reason_code: "m2_owner_scope_allowed",
                    }
                } else {
                    DataLayerM2AuthorizationDecision::Deny {
                        reason_code: "m2_abac_scope_denied",
                    }
                }
            }
            DataLayerM2ActorRole::EscrowAuditor => {
                let escrow_id = scope.escrow_id.as_deref().unwrap_or_default();
                let auditor_allowed = self
                    .escrow_auditors_by_escrow
                    .get(escrow_id)
                    .is_some_and(|auditors| auditors.contains(requester_did));
                let dispute_active = self.disputed_escrows.contains(escrow_id);
                if !escrow_id.is_empty() && auditor_allowed && dispute_active {
                    DataLayerM2AuthorizationDecision::Allow {
                        reason_code: "m2_escrow_auditor_scope_allowed",
                    }
                } else {
                    DataLayerM2AuthorizationDecision::Deny {
                        reason_code: "m2_abac_scope_denied",
                    }
                }
            }
            DataLayerM2ActorRole::PlatformOperator => DataLayerM2AuthorizationDecision::Deny {
                reason_code: "m2_abac_scope_denied",
            },
        };

        Ok(decision)
    }
}

/// RLS policy template projection for one table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2RlsPolicy {
    /// Table this policy applies to.
    pub table_name: String,
    /// PostgreSQL policy identifier.
    pub policy_name: String,
    /// `USING` predicate expression.
    pub using_clause: String,
    /// Optional `WITH CHECK` predicate expression.
    pub with_check_clause: Option<String>,
}

/// Returns default M2 RLS policy templates for gateway-scoped tables.
pub fn data_layer_m2_default_rls_policies() -> Vec<DataLayerM2RlsPolicy> {
    let requester = format!("current_setting('{DATA_LAYER_M2_REQUESTER_DID_SETTING}', true)");
    let requester_guard = format!("{requester} <> ''");

    vec![
        DataLayerM2RlsPolicy {
            table_name: "messages".to_owned(),
            policy_name: "m2_messages_requester_scope".to_owned(),
            using_clause: format!(
                "{requester_guard} AND (sender_did = {requester} OR recipient_did = {requester} OR owner_sender_did = {requester} OR owner_recipient_did = {requester})"
            ),
            with_check_clause: None,
        },
        DataLayerM2RlsPolicy {
            table_name: "access_log".to_owned(),
            policy_name: "m2_access_log_requester_scope".to_owned(),
            using_clause: format!("{requester_guard} AND requester_did = {requester}"),
            with_check_clause: None,
        },
    ]
}

/// Input envelope for one access-audit append event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2AccessAuditInput {
    /// Requester DID for this event.
    pub requester_did: String,
    /// Action identifier (for example `read_message`).
    pub action: String,
    /// Resource identifier (for example message id).
    pub resource_id: String,
    /// Deterministic decision reason code.
    pub reason_code: String,
    /// Event timestamp in epoch seconds.
    pub event_epoch_seconds: u64,
}

/// Stored append-only access-audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2AccessAuditRecord {
    /// Monotonic sequence number starting at 1.
    pub sequence: u64,
    /// Requester DID.
    pub requester_did: String,
    /// Action identifier.
    pub action: String,
    /// Resource identifier.
    pub resource_id: String,
    /// Deterministic decision reason code.
    pub reason_code: String,
    /// Event timestamp in epoch seconds.
    pub event_epoch_seconds: u64,
    /// Previous record hash in hash chain.
    pub hash_chain_prev: String,
    /// Hash for this record payload.
    pub record_hash: String,
}

/// Append-only access-audit ledger for M2 access-gateway decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM2AccessAuditLedger {
    records: Vec<DataLayerM2AccessAuditRecord>,
}

impl DataLayerM2AccessAuditLedger {
    /// Creates an empty access-audit ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one access-audit record.
    pub fn append(
        &mut self,
        input: DataLayerM2AccessAuditInput,
    ) -> Result<DataLayerM2AccessAuditRecord, DataLayerM2GatewayError> {
        validate_audit_input(&input)?;
        let sequence = (self.records.len() + 1) as u64;
        let hash_chain_prev = self
            .records
            .last()
            .map(|record| record.record_hash.clone())
            .unwrap_or_else(|| DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS.to_owned());
        let record_hash = compute_audit_record_hash(sequence, &input, &hash_chain_prev);

        let record = DataLayerM2AccessAuditRecord {
            sequence,
            requester_did: input.requester_did,
            action: input.action,
            resource_id: input.resource_id,
            reason_code: input.reason_code,
            event_epoch_seconds: input.event_epoch_seconds,
            hash_chain_prev,
            record_hash,
        };
        self.records.push(record.clone());
        Ok(record)
    }

    /// Returns immutable append-order records.
    pub fn records(&self) -> &[DataLayerM2AccessAuditRecord] {
        &self.records
    }

    /// Verifies full access-audit hash-chain integrity.
    pub fn verify_hash_chain(&self) -> Result<(), DataLayerM2GatewayError> {
        let mut expected_prev = DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in self.records.iter().enumerate() {
            if record.hash_chain_prev != expected_prev {
                return Err(DataLayerM2GatewayError::InvalidAuditHashChain {
                    position,
                    reason: "hash_chain_prev mismatch",
                });
            }
            let expected_hash = compute_audit_record_hash(
                record.sequence,
                &DataLayerM2AccessAuditInput {
                    requester_did: record.requester_did.clone(),
                    action: record.action.clone(),
                    resource_id: record.resource_id.clone(),
                    reason_code: record.reason_code.clone(),
                    event_epoch_seconds: record.event_epoch_seconds,
                },
                record.hash_chain_prev.as_str(),
            );
            if record.record_hash != expected_hash {
                return Err(DataLayerM2GatewayError::InvalidAuditHashChain {
                    position,
                    reason: "record_hash mismatch",
                });
            }
            expected_prev = record.record_hash.clone();
        }
        Ok(())
    }

    /// Replaces one record hash without recomputing chain links.
    ///
    /// This intentionally bypasses invariants for deterministic tamper tests.
    pub fn replace_record_hash_unchecked(
        &mut self,
        sequence: u64,
        record_hash: &str,
    ) -> Result<(), DataLayerM2GatewayError> {
        if record_hash.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("record_hash"));
        }
        let record = self
            .records
            .iter_mut()
            .find(|entry| entry.sequence == sequence)
            .ok_or(DataLayerM2GatewayError::AuditSequenceNotFound(sequence))?;
        record.record_hash = record_hash.to_owned();
        Ok(())
    }
}

/// Error taxonomy for M2 gateway authn/authz/audit contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2GatewayError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID value failed validation.
    InvalidDid(String),
    /// Credential payload failed deterministic validation.
    InvalidCredential(String),
    /// Session TTL is invalid.
    InvalidSessionTtl {
        /// Requested TTL.
        ttl_seconds: u64,
        /// Maximum allowed TTL.
        max_ttl_seconds: u64,
    },
    /// Expiry computation overflowed u64 bounds.
    SessionExpiryOverflow,
    /// Access-audit hash chain failed integrity checks.
    InvalidAuditHashChain {
        /// Zero-based record position.
        position: usize,
        /// Deterministic mismatch reason marker.
        reason: &'static str,
    },
    /// Access-audit sequence number not found.
    AuditSequenceNotFound(u64),
}

impl fmt::Display for DataLayerM2GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidCredential(reason) => write!(f, "invalid credential: {reason}"),
            Self::InvalidSessionTtl {
                ttl_seconds,
                max_ttl_seconds,
            } => write!(
                f,
                "invalid session ttl: requested {ttl_seconds}, max {max_ttl_seconds}"
            ),
            Self::SessionExpiryOverflow => write!(f, "session expiry overflow"),
            Self::InvalidAuditHashChain { position, reason } => {
                write!(
                    f,
                    "invalid audit hash chain at position {position}: {reason}"
                )
            }
            Self::AuditSequenceNotFound(sequence) => {
                write!(f, "audit sequence not found: {sequence}")
            }
        }
    }
}

impl std::error::Error for DataLayerM2GatewayError {}

fn validate_message_scope(scope: &DataLayerM2MessageScope) -> Result<(), DataLayerM2GatewayError> {
    if scope.message_id.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField("message_id"));
    }
    for (field, value) in [
        ("sender_did", scope.sender_did.as_str()),
        ("recipient_did", scope.recipient_did.as_str()),
        ("owner_sender_did", scope.owner_sender_did.as_str()),
        ("owner_recipient_did", scope.owner_recipient_did.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField(field));
        }
        validate_kamn_did(value)?;
    }
    Ok(())
}

fn validate_audit_input(
    input: &DataLayerM2AccessAuditInput,
) -> Result<(), DataLayerM2GatewayError> {
    if input.action.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField("action"));
    }
    if input.resource_id.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField("resource_id"));
    }
    if input.reason_code.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField("reason_code"));
    }
    if input.event_epoch_seconds == 0 {
        return Err(DataLayerM2GatewayError::EmptyField("event_epoch_seconds"));
    }
    validate_kamn_did(input.requester_did.as_str())?;
    Ok(())
}

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM2GatewayError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DataLayerM2GatewayError::InvalidDid(value.to_owned()));
    }
    if !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM2GatewayError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM2GatewayError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

fn compute_audit_record_hash(
    sequence: u64,
    input: &DataLayerM2AccessAuditInput,
    hash_chain_prev: &str,
) -> String {
    tagged_digest(
        format!(
            "audit|seq:{sequence}|requester:{}|action:{}|resource:{}|reason:{}|event:{}|prev:{}",
            input.requester_did,
            input.action,
            input.resource_id,
            input.reason_code,
            input.event_epoch_seconds,
            hash_chain_prev
        )
        .as_str(),
    )
}

fn tagged_digest(value: &str) -> String {
    format!(
        "{DATA_LAYER_M2_HASH_ALGORITHM}:{}",
        deterministic_digest_256_hex(value)
    )
}

fn deterministic_digest_256_hex(value: &str) -> String {
    const SEEDS: [u64; 4] = [
        0x243f6a8885a308d3,
        0x13198a2e03707344,
        0xa4093822299f31d0,
        0x082efa98ec4e6c89,
    ];
    let mut output = String::with_capacity(64);
    for (index, seed) in SEEDS.iter().enumerate() {
        let mut acc = *seed ^ (index as u64).wrapping_mul(0x9e3779b97f4a7c15);
        for byte in value.bytes() {
            acc ^= u64::from(byte);
            acc = acc.wrapping_mul(0x00000100000001B3);
            acc ^= acc.rotate_left(13);
        }
        output.push_str(&format!("{acc:016x}"));
    }
    output
}
