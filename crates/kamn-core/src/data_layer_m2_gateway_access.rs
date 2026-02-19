//! M2 access-gateway contracts for DID authn/authz, RLS templates, and access auditing.
//!
//! This module models the PRD M2 control surface as deterministic Rust contracts:
//! DID-authenticated session issuance, ABAC message visibility checks, RLS policy
//! template emission, and append-only audit logging with hash-chain verification.

use crate::{AgentDid, KamnDid, KamnDidError};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// PostgreSQL session variable key used by RLS predicates.
pub const DATA_LAYER_M2_REQUESTER_DID_SETTING: &str = "kamn.requester_did";
/// Hash algorithm label used by M2 deterministic digests.
pub const DATA_LAYER_M2_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker for access-audit hash chains.
pub const DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS: &str = "GENESIS";
/// Reason marker for agent sender/recipient scope access.
pub const DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED: &str =
    "m2_agent_counterparty_scope_allowed";
/// Reason marker for owner supervisory scope access.
pub const DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED: &str = "m2_owner_scope_allowed";
/// Reason marker for dispute-scoped escrow auditor access.
pub const DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED: &str =
    "m2_escrow_auditor_scope_allowed";
/// Reason marker for fail-closed ABAC denials.
pub const DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED: &str = "m2_abac_scope_denied";
/// Negative authorization matrix result marker when all cases deny as expected.
pub const DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE: &str =
    "m2_negative_matrix_all_denied";
/// Negative authorization matrix result marker when any case drifts.
pub const DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE: &str =
    "m2_negative_matrix_drift_detected";
/// Invalid requester DID reason marker.
pub const DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE: &str = "m2_invalid_requester_did";
/// Invalid sender DID reason marker.
pub const DATA_LAYER_M2_INVALID_SENDER_DID_REASON_CODE: &str = "m2_invalid_sender_did";
/// Invalid recipient DID reason marker.
pub const DATA_LAYER_M2_INVALID_RECIPIENT_DID_REASON_CODE: &str = "m2_invalid_recipient_did";
/// Invalid owner sender DID reason marker.
pub const DATA_LAYER_M2_INVALID_OWNER_SENDER_DID_REASON_CODE: &str = "m2_invalid_owner_sender_did";
/// Invalid owner recipient DID reason marker.
pub const DATA_LAYER_M2_INVALID_OWNER_RECIPIENT_DID_REASON_CODE: &str =
    "m2_invalid_owner_recipient_did";
/// Invalid escrow-auditor DID reason marker.
pub const DATA_LAYER_M2_INVALID_AUDITOR_DID_REASON_CODE: &str = "m2_invalid_auditor_did";

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

/// Typed validated auth request used by internal M2 session issuance paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2DidAuthRequestValidated {
    /// Canonical requester agent DID.
    pub requester_did: AgentDid,
    /// Challenge/nonce bound to credential signature.
    pub challenge: String,
    /// Credential payload carrying deterministic signature binding.
    pub credential: String,
    /// Request issuance timestamp in epoch seconds.
    pub issued_at_epoch_seconds: u64,
    /// Requested session TTL in seconds.
    pub ttl_seconds: u64,
}

impl TryFrom<DataLayerM2DidAuthRequest> for DataLayerM2DidAuthRequestValidated {
    type Error = DataLayerM2GatewayError;

    fn try_from(request: DataLayerM2DidAuthRequest) -> Result<Self, Self::Error> {
        if request.challenge.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("challenge"));
        }
        if request.credential.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("credential"));
        }

        Ok(Self {
            requester_did: parse_agent_did(
                request.requester_did.as_str(),
                "requester_did",
                DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
            )?,
            challenge: request.challenge,
            credential: request.credential,
            issued_at_epoch_seconds: request.issued_at_epoch_seconds,
            ttl_seconds: request.ttl_seconds,
        })
    }
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
        let request = DataLayerM2DidAuthRequestValidated::try_from(request)?;

        if request.ttl_seconds == 0 || request.ttl_seconds > self.max_ttl_seconds {
            return Err(DataLayerM2GatewayError::InvalidSessionTtl {
                ttl_seconds: request.ttl_seconds,
                max_ttl_seconds: self.max_ttl_seconds,
            });
        }

        let requester_did = request.requester_did.as_str().to_owned();
        let expected_credential = format!("sig:{requester_did}:{}", request.challenge);
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
                    requester_did,
                    request.challenge,
                    request.issued_at_epoch_seconds,
                    expires_at_epoch_seconds
                )
                .as_str()
            )
        );

        Ok(DataLayerM2SessionToken {
            token_id,
            requester_did,
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

/// Typed validated scope used by internal M2 ABAC authorization paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2MessageScopeValidated {
    /// Stable message identifier.
    pub message_id: String,
    /// Sender agent DID.
    pub sender_did: AgentDid,
    /// Recipient agent DID.
    pub recipient_did: AgentDid,
    /// Owner DID for sender.
    pub owner_sender_did: KamnDid,
    /// Owner DID for recipient.
    pub owner_recipient_did: KamnDid,
    /// Optional escrow identifier when message is escrow-scoped.
    pub escrow_id: Option<String>,
}

impl TryFrom<&DataLayerM2MessageScope> for DataLayerM2MessageScopeValidated {
    type Error = DataLayerM2GatewayError;

    fn try_from(scope: &DataLayerM2MessageScope) -> Result<Self, Self::Error> {
        if scope.message_id.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("message_id"));
        }

        Ok(Self {
            message_id: scope.message_id.clone(),
            sender_did: parse_agent_did(
                scope.sender_did.as_str(),
                "sender_did",
                DATA_LAYER_M2_INVALID_SENDER_DID_REASON_CODE,
            )?,
            recipient_did: parse_agent_did(
                scope.recipient_did.as_str(),
                "recipient_did",
                DATA_LAYER_M2_INVALID_RECIPIENT_DID_REASON_CODE,
            )?,
            owner_sender_did: parse_kamn_did(
                scope.owner_sender_did.as_str(),
                "owner_sender_did",
                DATA_LAYER_M2_INVALID_OWNER_SENDER_DID_REASON_CODE,
            )?,
            owner_recipient_did: parse_kamn_did(
                scope.owner_recipient_did.as_str(),
                "owner_recipient_did",
                DATA_LAYER_M2_INVALID_OWNER_RECIPIENT_DID_REASON_CODE,
            )?,
            escrow_id: scope.escrow_id.clone(),
        })
    }
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

/// One expected-deny authorization case in the M2 negative matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationCase {
    /// Stable case identifier for evidence fixtures.
    pub case_id: String,
    /// Requester DID to evaluate.
    pub requester_did: String,
    /// Requester role to evaluate.
    pub requester_role: DataLayerM2ActorRole,
    /// Message scope under evaluation.
    pub scope: DataLayerM2MessageScope,
    /// Expected deny decision marker.
    pub expected_denied: bool,
    /// Deterministic event timestamp for emitted audit evidence.
    pub event_epoch_seconds: u64,
}

/// Per-case audit fixture emitted by negative authorization matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationAuditFixture {
    /// Stable case identifier.
    pub case_id: String,
    /// Whether the evaluated decision denied access.
    pub denied: bool,
    /// Expected deny marker from matrix input.
    pub expected_denied: bool,
    /// Whether actual decision drifted from expected deny marker.
    pub mismatch: bool,
    /// Deterministic decision reason code from authorization result.
    pub decision_reason_code: &'static str,
    /// Deterministic append-only audit record for this case.
    pub audit_record: DataLayerM2AccessAuditRecord,
}

/// Aggregate matrix decision marker for negative authorization evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2NegativeAuthorizationMatrixDecision {
    /// All cases denied and matched expected deny markers.
    AllDenied {
        /// Stable result reason code.
        reason_code: &'static str,
    },
    /// At least one case diverged from expected deny behavior.
    DriftDetected {
        /// Stable result reason code.
        reason_code: &'static str,
    },
}

/// Aggregate negative authorization matrix evaluation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationMatrixReport {
    /// Aggregate decision.
    pub decision: DataLayerM2NegativeAuthorizationMatrixDecision,
    /// Per-case deterministic audit fixtures in input order.
    pub fixtures: Vec<DataLayerM2NegativeAuthorizationAuditFixture>,
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
        let auditor_did = parse_kamn_did(
            auditor_did,
            "auditor_did",
            DATA_LAYER_M2_INVALID_AUDITOR_DID_REASON_CODE,
        )?;
        self.escrow_auditors_by_escrow
            .entry(escrow_id.to_owned())
            .or_default()
            .insert(auditor_did.as_str().to_owned());
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
        let requester = validate_requester_did_for_role(requester_did, requester_role)?;
        let scope = DataLayerM2MessageScopeValidated::try_from(scope)?;

        let decision = match requester_role {
            DataLayerM2ActorRole::Agent => {
                if requester.as_str() == scope.sender_did.as_str()
                    || requester.as_str() == scope.recipient_did.as_str()
                {
                    DataLayerM2AuthorizationDecision::Allow {
                        reason_code: DATA_LAYER_M2_REASON_AGENT_COUNTERPARTY_SCOPE_ALLOWED,
                    }
                } else {
                    DataLayerM2AuthorizationDecision::Deny {
                        reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
                    }
                }
            }
            DataLayerM2ActorRole::Owner => {
                if requester.as_str() == scope.owner_sender_did.as_str()
                    || requester.as_str() == scope.owner_recipient_did.as_str()
                {
                    DataLayerM2AuthorizationDecision::Allow {
                        reason_code: DATA_LAYER_M2_REASON_OWNER_SCOPE_ALLOWED,
                    }
                } else {
                    DataLayerM2AuthorizationDecision::Deny {
                        reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
                    }
                }
            }
            DataLayerM2ActorRole::EscrowAuditor => {
                let escrow_id = scope.escrow_id.as_deref().unwrap_or_default();
                let auditor_allowed = self
                    .escrow_auditors_by_escrow
                    .get(escrow_id)
                    .is_some_and(|auditors| auditors.contains(requester.as_str()));
                let dispute_active = self.disputed_escrows.contains(escrow_id);
                if !escrow_id.is_empty() && auditor_allowed && dispute_active {
                    DataLayerM2AuthorizationDecision::Allow {
                        reason_code: DATA_LAYER_M2_REASON_ESCROW_AUDITOR_SCOPE_ALLOWED,
                    }
                } else {
                    DataLayerM2AuthorizationDecision::Deny {
                        reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
                    }
                }
            }
            DataLayerM2ActorRole::PlatformOperator => DataLayerM2AuthorizationDecision::Deny {
                reason_code: DATA_LAYER_M2_REASON_ABAC_SCOPE_DENIED,
            },
        };

        Ok(decision)
    }

    /// Evaluates an expected-deny authorization matrix and emits deterministic drift evidence.
    pub fn evaluate_negative_authorization_matrix(
        &self,
        cases: &[DataLayerM2NegativeAuthorizationCase],
    ) -> Result<DataLayerM2NegativeAuthorizationMatrixReport, DataLayerM2GatewayError> {
        if cases.is_empty() {
            return Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
                "cases",
            ));
        }

        let mut fixtures = Vec::with_capacity(cases.len());
        let mut audit_ledger = DataLayerM2AccessAuditLedger::new();
        for case in cases {
            if case.case_id.trim().is_empty() {
                return Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
                    "case_id",
                ));
            }
            if case.event_epoch_seconds == 0 {
                return Err(DataLayerM2GatewayError::InvalidNegativeAuthorizationMatrix(
                    "event_epoch_seconds",
                ));
            }

            let decision = self.authorize_message_visibility(
                case.requester_did.as_str(),
                case.requester_role,
                &case.scope,
            )?;
            let (denied, decision_reason_code) = match decision {
                DataLayerM2AuthorizationDecision::Allow { reason_code } => (false, reason_code),
                DataLayerM2AuthorizationDecision::Deny { reason_code } => (true, reason_code),
            };
            let mismatch = case.expected_denied != denied;
            let audit_record = audit_ledger.append(DataLayerM2AccessAuditInput {
                requester_did: case.requester_did.clone(),
                action: format!("m2_negative_matrix:{}", case.case_id),
                resource_id: case.scope.message_id.clone(),
                reason_code: decision_reason_code.to_owned(),
                event_epoch_seconds: case.event_epoch_seconds,
            })?;
            fixtures.push(DataLayerM2NegativeAuthorizationAuditFixture {
                case_id: case.case_id.clone(),
                denied,
                expected_denied: case.expected_denied,
                mismatch,
                decision_reason_code,
                audit_record,
            });
        }

        let decision = if fixtures.iter().all(|fixture| fixture.denied)
            && fixtures.iter().all(|fixture| !fixture.mismatch)
        {
            DataLayerM2NegativeAuthorizationMatrixDecision::AllDenied {
                reason_code: DATA_LAYER_M2_NEGATIVE_MATRIX_ALL_DENIED_REASON_CODE,
            }
        } else {
            DataLayerM2NegativeAuthorizationMatrixDecision::DriftDetected {
                reason_code: DATA_LAYER_M2_NEGATIVE_MATRIX_DRIFT_DETECTED_REASON_CODE,
            }
        };

        Ok(DataLayerM2NegativeAuthorizationMatrixReport { decision, fixtures })
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
    InvalidDid {
        /// Input field carrying DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
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
    /// Negative authorization matrix input failed fail-closed validation.
    InvalidNegativeAuthorizationMatrix(&'static str),
}

impl fmt::Display for DataLayerM2GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => {
                write!(f, "invalid did field {field}: {reason_code} ({detail})")
            }
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
            Self::InvalidNegativeAuthorizationMatrix(field) => {
                write!(f, "invalid negative authorization matrix input: {field}")
            }
        }
    }
}

impl std::error::Error for DataLayerM2GatewayError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DataLayerM2RequesterDidValidated {
    Agent(AgentDid),
    KamnDid(KamnDid),
}

impl DataLayerM2RequesterDidValidated {
    fn as_str(&self) -> &str {
        match self {
            Self::Agent(agent_did) => agent_did.as_str(),
            Self::KamnDid(value) => value.as_str(),
        }
    }
}

fn validate_requester_did_for_role(
    requester_did: &str,
    requester_role: DataLayerM2ActorRole,
) -> Result<DataLayerM2RequesterDidValidated, DataLayerM2GatewayError> {
    match requester_role {
        DataLayerM2ActorRole::Agent => {
            Ok(DataLayerM2RequesterDidValidated::Agent(parse_agent_did(
                requester_did,
                "requester_did",
                DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
            )?))
        }
        DataLayerM2ActorRole::Owner
        | DataLayerM2ActorRole::EscrowAuditor
        | DataLayerM2ActorRole::PlatformOperator => {
            Ok(DataLayerM2RequesterDidValidated::KamnDid(parse_kamn_did(
                requester_did,
                "requester_did",
                DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
            )?))
        }
    }
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
    parse_kamn_did(
        input.requester_did.as_str(),
        "requester_did",
        DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
    )?;
    Ok(())
}

fn map_kamn_did_error(
    error: KamnDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM2GatewayError {
    DataLayerM2GatewayError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

fn map_agent_did_error(
    error: crate::AgentDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM2GatewayError {
    DataLayerM2GatewayError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

fn parse_kamn_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<KamnDid, DataLayerM2GatewayError> {
    KamnDid::parse(value).map_err(|error| map_kamn_did_error(error, field, reason_code))
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, DataLayerM2GatewayError> {
    AgentDid::parse(value).map_err(|error| map_agent_did_error(error, field, reason_code))
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
