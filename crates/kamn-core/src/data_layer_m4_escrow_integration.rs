//! M4 escrow integration contracts for state transitions, scoped visibility, and settlement evidence.
//!
//! This module models the PRD M4 escrow surface as deterministic Rust contracts:
//! escrow lifecycle transitions, dispute-aware participant/auditor message visibility,
//! and append-only settlement evidence storage with hash-chain verification.

use std::collections::BTreeMap;
use std::fmt;

/// Hash algorithm label used by M4 deterministic digests.
pub const DATA_LAYER_M4_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker used by per-escrow settlement evidence hash chains.
pub const DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS: &str = "GENESIS";

/// Escrow state projection aligned to PRD M4 lifecycle markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM4EscrowState {
    /// Escrow draft created but not funded.
    Created,
    /// Escrow funding confirmed.
    Funded,
    /// Escrow active for normal operations.
    Active,
    /// Escrow in dispute state.
    Disputed,
    /// Escrow settled by release.
    Released,
    /// Escrow settled by refund.
    Refunded,
    /// Escrow expired without settlement.
    Expired,
}

impl DataLayerM4EscrowState {
    fn as_marker(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Funded => "funded",
            Self::Active => "active",
            Self::Disputed => "disputed",
            Self::Released => "released",
            Self::Refunded => "refunded",
            Self::Expired => "expired",
        }
    }
}

/// Input for creating one escrow draft record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowDraftInput {
    /// Stable escrow identifier.
    pub escrow_id: String,
    /// Initiator DID.
    pub initiator_did: String,
    /// Counterparty DID.
    pub counterparty_did: String,
    /// Optional escrow auditor DID.
    pub auditor_did: Option<String>,
    /// Optional threshold shares required for auditor reconstruction.
    pub auditor_threshold: Option<u8>,
    /// DIDs of share holders for auditor reconstruction.
    pub auditor_share_holders: Vec<String>,
    /// Optional expiration timestamp.
    pub expires_at_epoch_seconds: Option<u64>,
}

/// Stored escrow projection managed by the M4 transition engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowRecord {
    /// Stable escrow identifier.
    pub escrow_id: String,
    /// Initiator DID.
    pub initiator_did: String,
    /// Counterparty DID.
    pub counterparty_did: String,
    /// Optional escrow auditor DID.
    pub auditor_did: Option<String>,
    /// Optional threshold shares required for auditor reconstruction.
    pub auditor_threshold: Option<u8>,
    /// DIDs of share holders for auditor reconstruction.
    pub auditor_share_holders: Vec<String>,
    /// Current escrow state.
    pub state: DataLayerM4EscrowState,
    /// Optional expiration timestamp.
    pub expires_at_epoch_seconds: Option<u64>,
    /// Optional dispute-opened timestamp.
    pub dispute_opened_at_epoch_seconds: Option<u64>,
    /// Optional settlement timestamp.
    pub settled_at_epoch_seconds: Option<u64>,
    /// Optional settlement receipt hash for final states.
    pub settlement_receipt_hash: Option<String>,
}

/// State transition action for one escrow record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4EscrowTransitionAction {
    /// Move `Created -> Funded`.
    Fund {
        /// Funding timestamp.
        funded_at_epoch_seconds: u64,
    },
    /// Move `Funded -> Active`.
    Activate {
        /// Activation timestamp.
        activated_at_epoch_seconds: u64,
    },
    /// Move `Active -> Disputed`.
    OpenDispute {
        /// Dispute-opened timestamp.
        dispute_opened_at_epoch_seconds: u64,
    },
    /// Move `Active|Disputed -> Released`.
    ResolveRelease {
        /// Settlement timestamp.
        settled_at_epoch_seconds: u64,
        /// Settlement receipt hash.
        settlement_receipt_hash: String,
    },
    /// Move `Active|Disputed -> Refunded`.
    ResolveRefund {
        /// Settlement timestamp.
        settled_at_epoch_seconds: u64,
        /// Settlement receipt hash.
        settlement_receipt_hash: String,
    },
    /// Move `Created|Funded|Active -> Expired`.
    Expire {
        /// Expiration timestamp.
        expired_at_epoch_seconds: u64,
    },
}

impl DataLayerM4EscrowTransitionAction {
    fn marker(&self) -> &'static str {
        match self {
            Self::Fund { .. } => "fund",
            Self::Activate { .. } => "activate",
            Self::OpenDispute { .. } => "open_dispute",
            Self::ResolveRelease { .. } => "resolve_release",
            Self::ResolveRefund { .. } => "resolve_refund",
            Self::Expire { .. } => "expire",
        }
    }
}

/// Transition evidence projected for successful escrow state mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowTransitionEvidence {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Previous state.
    pub from: DataLayerM4EscrowState,
    /// Action applied.
    pub action: DataLayerM4EscrowTransitionAction,
    /// Resulting state.
    pub to: DataLayerM4EscrowState,
    /// Stable reason code marker.
    pub reason_code: &'static str,
}

/// Visibility request for one escrow-scoped message lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowVisibilityRequest {
    /// Escrow identifier being accessed.
    pub escrow_id: String,
    /// Requester DID.
    pub requester_did: String,
    /// Optional number of reconstructed auditor shares presented by requester.
    pub reconstructed_auditor_shares: Option<u8>,
}

/// Visibility decision for escrow-scoped message lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4EscrowVisibilityDecision {
    /// Access allowed with reason code.
    Allow {
        /// Stable reason code marker.
        reason_code: &'static str,
    },
    /// Access denied with reason code.
    Deny {
        /// Stable reason code marker.
        reason_code: &'static str,
    },
}

/// Transition and visibility engine for M4 escrow records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM4EscrowTransitionEngine {
    escrows: BTreeMap<String, DataLayerM4EscrowRecord>,
}

impl DataLayerM4EscrowTransitionEngine {
    /// Creates an empty escrow transition engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates and stores one escrow in `Created` state.
    pub fn create_escrow(
        &mut self,
        input: DataLayerM4EscrowDraftInput,
    ) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
        validate_non_empty(input.escrow_id.as_str(), "escrow_id")?;
        validate_kamn_did(input.initiator_did.as_str())?;
        validate_kamn_did(input.counterparty_did.as_str())?;
        if input.initiator_did == input.counterparty_did {
            return Err(
                DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowParties(
                    "initiator and counterparty must be distinct",
                ),
            );
        }

        if let Some(auditor_did) = input.auditor_did.as_deref() {
            validate_kamn_did(auditor_did)?;
        }
        for holder in &input.auditor_share_holders {
            validate_kamn_did(holder.as_str())?;
        }

        validate_auditor_threshold(
            input.auditor_did.as_ref(),
            input.auditor_threshold,
            input.auditor_share_holders.len(),
        )?;

        if let Some(expires_at_epoch_seconds) = input.expires_at_epoch_seconds {
            if expires_at_epoch_seconds == 0 {
                return Err(DataLayerM4SettlementEvidenceRegistryError::EmptyField(
                    "expires_at_epoch_seconds",
                ));
            }
        }

        if self.escrows.contains_key(input.escrow_id.as_str()) {
            return Err(
                DataLayerM4SettlementEvidenceRegistryError::DuplicateEscrowId(input.escrow_id),
            );
        }

        let escrow = DataLayerM4EscrowRecord {
            escrow_id: input.escrow_id.clone(),
            initiator_did: input.initiator_did,
            counterparty_did: input.counterparty_did,
            auditor_did: input.auditor_did,
            auditor_threshold: input.auditor_threshold,
            auditor_share_holders: input.auditor_share_holders,
            state: DataLayerM4EscrowState::Created,
            expires_at_epoch_seconds: input.expires_at_epoch_seconds,
            dispute_opened_at_epoch_seconds: None,
            settled_at_epoch_seconds: None,
            settlement_receipt_hash: None,
        };
        self.escrows.insert(escrow.escrow_id.clone(), escrow);
        Ok(())
    }

    /// Returns one stored escrow record by identifier.
    pub fn escrow(&self, escrow_id: &str) -> Option<&DataLayerM4EscrowRecord> {
        self.escrows.get(escrow_id)
    }

    /// Applies one escrow transition action.
    pub fn apply_transition(
        &mut self,
        escrow_id: &str,
        action: DataLayerM4EscrowTransitionAction,
    ) -> Result<DataLayerM4EscrowTransitionEvidence, DataLayerM4SettlementEvidenceRegistryError>
    {
        validate_non_empty(escrow_id, "escrow_id")?;
        let escrow = self.escrows.get_mut(escrow_id).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: escrow_id.to_owned(),
            }
        })?;

        let from = escrow.state;
        let to = match action.clone() {
            DataLayerM4EscrowTransitionAction::Fund {
                funded_at_epoch_seconds,
            } => {
                validate_non_zero_timestamp(funded_at_epoch_seconds, "funded_at_epoch_seconds")?;
                ensure_transition_allowed(
                    escrow.escrow_id.as_str(),
                    from,
                    &action,
                    &[DataLayerM4EscrowState::Created],
                )?;
                DataLayerM4EscrowState::Funded
            }
            DataLayerM4EscrowTransitionAction::Activate {
                activated_at_epoch_seconds,
            } => {
                validate_non_zero_timestamp(
                    activated_at_epoch_seconds,
                    "activated_at_epoch_seconds",
                )?;
                ensure_transition_allowed(
                    escrow.escrow_id.as_str(),
                    from,
                    &action,
                    &[DataLayerM4EscrowState::Funded],
                )?;
                DataLayerM4EscrowState::Active
            }
            DataLayerM4EscrowTransitionAction::OpenDispute {
                dispute_opened_at_epoch_seconds,
            } => {
                validate_non_zero_timestamp(
                    dispute_opened_at_epoch_seconds,
                    "dispute_opened_at_epoch_seconds",
                )?;
                ensure_transition_allowed(
                    escrow.escrow_id.as_str(),
                    from,
                    &action,
                    &[DataLayerM4EscrowState::Active],
                )?;
                escrow.dispute_opened_at_epoch_seconds = Some(dispute_opened_at_epoch_seconds);
                DataLayerM4EscrowState::Disputed
            }
            DataLayerM4EscrowTransitionAction::ResolveRelease {
                settled_at_epoch_seconds,
                settlement_receipt_hash,
            } => {
                validate_non_zero_timestamp(settled_at_epoch_seconds, "settled_at_epoch_seconds")?;
                validate_hash_token(settlement_receipt_hash.as_str(), "settlement_receipt_hash")?;
                ensure_transition_allowed(
                    escrow.escrow_id.as_str(),
                    from,
                    &action,
                    &[
                        DataLayerM4EscrowState::Active,
                        DataLayerM4EscrowState::Disputed,
                    ],
                )?;
                escrow.settled_at_epoch_seconds = Some(settled_at_epoch_seconds);
                escrow.settlement_receipt_hash = Some(settlement_receipt_hash);
                DataLayerM4EscrowState::Released
            }
            DataLayerM4EscrowTransitionAction::ResolveRefund {
                settled_at_epoch_seconds,
                settlement_receipt_hash,
            } => {
                validate_non_zero_timestamp(settled_at_epoch_seconds, "settled_at_epoch_seconds")?;
                validate_hash_token(settlement_receipt_hash.as_str(), "settlement_receipt_hash")?;
                ensure_transition_allowed(
                    escrow.escrow_id.as_str(),
                    from,
                    &action,
                    &[
                        DataLayerM4EscrowState::Active,
                        DataLayerM4EscrowState::Disputed,
                    ],
                )?;
                escrow.settled_at_epoch_seconds = Some(settled_at_epoch_seconds);
                escrow.settlement_receipt_hash = Some(settlement_receipt_hash);
                DataLayerM4EscrowState::Refunded
            }
            DataLayerM4EscrowTransitionAction::Expire {
                expired_at_epoch_seconds,
            } => {
                validate_non_zero_timestamp(expired_at_epoch_seconds, "expired_at_epoch_seconds")?;
                ensure_transition_allowed(
                    escrow.escrow_id.as_str(),
                    from,
                    &action,
                    &[
                        DataLayerM4EscrowState::Created,
                        DataLayerM4EscrowState::Funded,
                        DataLayerM4EscrowState::Active,
                    ],
                )?;
                DataLayerM4EscrowState::Expired
            }
        };

        escrow.state = to;
        Ok(DataLayerM4EscrowTransitionEvidence {
            escrow_id: escrow.escrow_id.clone(),
            from,
            action: action.clone(),
            to,
            reason_code: reason_code_for_transition(&action),
        })
    }

    /// Evaluates requester visibility for escrow-scoped messages.
    pub fn authorize_message_visibility(
        &self,
        request: DataLayerM4EscrowVisibilityRequest,
    ) -> Result<DataLayerM4EscrowVisibilityDecision, DataLayerM4SettlementEvidenceRegistryError>
    {
        validate_non_empty(request.escrow_id.as_str(), "escrow_id")?;
        validate_kamn_did(request.requester_did.as_str())?;
        let escrow = self.escrows.get(request.escrow_id.as_str()).ok_or(
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: request.escrow_id,
            },
        )?;

        if request.requester_did == escrow.initiator_did
            || request.requester_did == escrow.counterparty_did
        {
            return Ok(DataLayerM4EscrowVisibilityDecision::Allow {
                reason_code: "m4_escrow_participant_scope_allowed",
            });
        }

        if escrow.auditor_did.as_deref() == Some(request.requester_did.as_str()) {
            if escrow.state != DataLayerM4EscrowState::Disputed {
                return Ok(DataLayerM4EscrowVisibilityDecision::Deny {
                    reason_code: "m4_escrow_auditor_dispute_required",
                });
            }
            let threshold = escrow.auditor_threshold.unwrap_or(0);
            if threshold == 0 {
                return Ok(DataLayerM4EscrowVisibilityDecision::Deny {
                    reason_code: "m4_escrow_auditor_threshold_not_configured",
                });
            }
            let shares = request.reconstructed_auditor_shares.unwrap_or(0);
            if shares >= threshold {
                return Ok(DataLayerM4EscrowVisibilityDecision::Allow {
                    reason_code: "m4_escrow_auditor_scope_allowed",
                });
            }
            return Ok(DataLayerM4EscrowVisibilityDecision::Deny {
                reason_code: "m4_escrow_auditor_threshold_not_met",
            });
        }

        Ok(DataLayerM4EscrowVisibilityDecision::Deny {
            reason_code: "m4_escrow_scope_denied",
        })
    }
}

/// Input envelope for one settlement evidence append event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4SettlementEvidenceInput {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Escrow terminal settlement state.
    pub escrow_state: DataLayerM4EscrowState,
    /// Settlement receipt hash marker.
    pub settlement_receipt_hash: String,
    /// Settlement payload hash marker.
    pub settlement_payload_hash: String,
    /// Recorded timestamp in epoch seconds.
    pub recorded_at_epoch_seconds: u64,
}

/// Stored append-only settlement evidence record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4SettlementEvidenceRecord {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Sequence number for this escrow starting at 1.
    pub sequence: u64,
    /// Escrow terminal settlement state.
    pub escrow_state: DataLayerM4EscrowState,
    /// Settlement receipt hash marker.
    pub settlement_receipt_hash: String,
    /// Settlement payload hash marker.
    pub settlement_payload_hash: String,
    /// Recorded timestamp in epoch seconds.
    pub recorded_at_epoch_seconds: u64,
    /// Previous hash-chain marker.
    pub hash_chain_prev: String,
    /// Hash for this evidence record.
    pub record_hash: String,
}

/// Append-only settlement evidence registry keyed by escrow identifier.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM4SettlementEvidenceRegistry {
    records_by_escrow: BTreeMap<String, Vec<DataLayerM4SettlementEvidenceRecord>>,
}

impl DataLayerM4SettlementEvidenceRegistry {
    /// Creates an empty settlement evidence registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one settlement evidence record.
    pub fn append(
        &mut self,
        input: DataLayerM4SettlementEvidenceInput,
    ) -> Result<DataLayerM4SettlementEvidenceRecord, DataLayerM4SettlementEvidenceRegistryError>
    {
        validate_non_empty(input.escrow_id.as_str(), "escrow_id")?;
        validate_non_zero_timestamp(input.recorded_at_epoch_seconds, "recorded_at_epoch_seconds")?;
        validate_hash_token(
            input.settlement_receipt_hash.as_str(),
            "settlement_receipt_hash",
        )?;
        validate_hash_token(
            input.settlement_payload_hash.as_str(),
            "settlement_payload_hash",
        )?;

        if input.escrow_state != DataLayerM4EscrowState::Released
            && input.escrow_state != DataLayerM4EscrowState::Refunded
        {
            return Err(
                DataLayerM4SettlementEvidenceRegistryError::UnsupportedSettlementState(
                    input.escrow_state,
                ),
            );
        }

        let escrow_records = self
            .records_by_escrow
            .entry(input.escrow_id.clone())
            .or_default();
        let sequence = (escrow_records.len() + 1) as u64;
        let hash_chain_prev = escrow_records
            .last()
            .map(|entry| entry.record_hash.clone())
            .unwrap_or_else(|| DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS.to_owned());
        let record_hash = compute_evidence_hash(sequence, &input, hash_chain_prev.as_str());

        let record = DataLayerM4SettlementEvidenceRecord {
            escrow_id: input.escrow_id,
            sequence,
            escrow_state: input.escrow_state,
            settlement_receipt_hash: input.settlement_receipt_hash,
            settlement_payload_hash: input.settlement_payload_hash,
            recorded_at_epoch_seconds: input.recorded_at_epoch_seconds,
            hash_chain_prev,
            record_hash,
        };
        escrow_records.push(record.clone());
        Ok(record)
    }

    /// Verifies evidence hash-chain integrity for one escrow.
    pub fn verify_escrow_integrity(
        &self,
        escrow_id: &str,
    ) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
        validate_non_empty(escrow_id, "escrow_id")?;
        let records = self.records_by_escrow.get(escrow_id).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: escrow_id.to_owned(),
            }
        })?;

        let mut expected_prev = DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in records.iter().enumerate() {
            if record.hash_chain_prev != expected_prev {
                return Err(
                    DataLayerM4SettlementEvidenceRegistryError::InvalidEvidenceHashChain {
                        escrow_id: escrow_id.to_owned(),
                        position,
                        reason: "hash_chain_prev mismatch",
                    },
                );
            }

            let expected_hash = compute_evidence_hash(
                record.sequence,
                &DataLayerM4SettlementEvidenceInput {
                    escrow_id: record.escrow_id.clone(),
                    escrow_state: record.escrow_state,
                    settlement_receipt_hash: record.settlement_receipt_hash.clone(),
                    settlement_payload_hash: record.settlement_payload_hash.clone(),
                    recorded_at_epoch_seconds: record.recorded_at_epoch_seconds,
                },
                record.hash_chain_prev.as_str(),
            );
            if record.record_hash != expected_hash {
                return Err(
                    DataLayerM4SettlementEvidenceRegistryError::InvalidEvidenceHashChain {
                        escrow_id: escrow_id.to_owned(),
                        position,
                        reason: "record_hash mismatch",
                    },
                );
            }
            expected_prev = record.record_hash.clone();
        }
        Ok(())
    }

    /// Replaces one record hash without recomputing chain links.
    ///
    /// This helper intentionally bypasses integrity checks for tamper regression tests.
    pub fn replace_record_hash_unchecked(
        &mut self,
        escrow_id: &str,
        sequence: u64,
        record_hash: &str,
    ) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
        validate_non_empty(escrow_id, "escrow_id")?;
        validate_non_empty(record_hash, "record_hash")?;
        let records = self.records_by_escrow.get_mut(escrow_id).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: escrow_id.to_owned(),
            }
        })?;
        let record = records
            .iter_mut()
            .find(|entry| entry.sequence == sequence)
            .ok_or_else(|| {
                DataLayerM4SettlementEvidenceRegistryError::EvidenceSequenceNotFound {
                    escrow_id: escrow_id.to_owned(),
                    sequence,
                }
            })?;
        record.record_hash = record_hash.to_owned();
        Ok(())
    }
}

/// Error taxonomy for M4 escrow transition/visibility/evidence contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4SettlementEvidenceRegistryError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Escrow participant relationship is invalid.
    InvalidEscrowParties(&'static str),
    /// Escrow identifier already exists.
    DuplicateEscrowId(String),
    /// Escrow identifier was not found.
    EscrowNotFound {
        /// Missing escrow identifier.
        escrow_id: String,
    },
    /// Transition was not allowed for current escrow state.
    InvalidEscrowTransition {
        /// Escrow identifier.
        escrow_id: String,
        /// Current state.
        from: DataLayerM4EscrowState,
        /// Attempted action marker.
        action: &'static str,
    },
    /// Auditor threshold configuration is invalid.
    InvalidAuditorThreshold {
        /// Provided threshold.
        threshold: u8,
        /// Number of configured share holders.
        share_holder_count: usize,
    },
    /// Hash token was malformed for a field.
    InvalidHashToken(&'static str),
    /// Settlement evidence append attempted for non-terminal state.
    UnsupportedSettlementState(DataLayerM4EscrowState),
    /// Evidence hash-chain integrity failed.
    InvalidEvidenceHashChain {
        /// Escrow identifier.
        escrow_id: String,
        /// Zero-based record position.
        position: usize,
        /// Mismatch reason marker.
        reason: &'static str,
    },
    /// Requested sequence number was not found.
    EvidenceSequenceNotFound {
        /// Escrow identifier.
        escrow_id: String,
        /// Missing sequence.
        sequence: u64,
    },
}

impl fmt::Display for DataLayerM4SettlementEvidenceRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidEscrowParties(reason) => write!(f, "invalid escrow parties: {reason}"),
            Self::DuplicateEscrowId(escrow_id) => write!(f, "duplicate escrow_id: {escrow_id}"),
            Self::EscrowNotFound { escrow_id } => write!(f, "escrow not found: {escrow_id}"),
            Self::InvalidEscrowTransition {
                escrow_id,
                from,
                action,
            } => write!(
                f,
                "invalid escrow transition for {escrow_id}: from {:?} via {action}",
                from
            ),
            Self::InvalidAuditorThreshold {
                threshold,
                share_holder_count,
            } => write!(
                f,
                "invalid auditor threshold {threshold} for {share_holder_count} share holders"
            ),
            Self::InvalidHashToken(field) => write!(f, "invalid hash token: {field}"),
            Self::UnsupportedSettlementState(state) => {
                write!(f, "unsupported settlement state: {:?}", state)
            }
            Self::InvalidEvidenceHashChain {
                escrow_id,
                position,
                reason,
            } => write!(
                f,
                "invalid evidence hash chain for {escrow_id} at position {position}: {reason}"
            ),
            Self::EvidenceSequenceNotFound {
                escrow_id,
                sequence,
            } => write!(f, "evidence sequence not found for {escrow_id}: {sequence}"),
        }
    }
}

impl std::error::Error for DataLayerM4SettlementEvidenceRegistryError {}

fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if value.trim().is_empty() {
        return Err(DataLayerM4SettlementEvidenceRegistryError::EmptyField(
            field_name,
        ));
    }
    Ok(())
}

fn validate_non_zero_timestamp(
    value: u64,
    field_name: &'static str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if value == 0 {
        return Err(DataLayerM4SettlementEvidenceRegistryError::EmptyField(
            field_name,
        ));
    }
    Ok(())
}

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidDid(
            value.to_owned(),
        ));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidDid(
            value.to_owned(),
        ));
    }
    Ok(())
}

fn validate_hash_token(
    hash: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    let trimmed = hash.trim();
    if trimmed.is_empty() || !trimmed.starts_with("sha256:") {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidHashToken(field_name));
    }
    Ok(())
}

fn validate_auditor_threshold(
    auditor_did: Option<&String>,
    threshold: Option<u8>,
    share_holder_count: usize,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if let Some(threshold) = threshold {
        if threshold == 0 || auditor_did.is_none() || share_holder_count < threshold as usize {
            return Err(
                DataLayerM4SettlementEvidenceRegistryError::InvalidAuditorThreshold {
                    threshold,
                    share_holder_count,
                },
            );
        }
    } else if auditor_did.is_some() && share_holder_count > 0 {
        return Err(
            DataLayerM4SettlementEvidenceRegistryError::InvalidAuditorThreshold {
                threshold: 0,
                share_holder_count,
            },
        );
    }
    Ok(())
}

fn ensure_transition_allowed(
    escrow_id: &str,
    from: DataLayerM4EscrowState,
    action: &DataLayerM4EscrowTransitionAction,
    allowed_states: &[DataLayerM4EscrowState],
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if !allowed_states.contains(&from) {
        return Err(
            DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowTransition {
                escrow_id: escrow_id.to_owned(),
                from,
                action: action.marker(),
            },
        );
    }
    Ok(())
}

fn reason_code_for_transition(action: &DataLayerM4EscrowTransitionAction) -> &'static str {
    match action {
        DataLayerM4EscrowTransitionAction::Fund { .. } => "m4_escrow_funded",
        DataLayerM4EscrowTransitionAction::Activate { .. } => "m4_escrow_active",
        DataLayerM4EscrowTransitionAction::OpenDispute { .. } => "m4_escrow_disputed",
        DataLayerM4EscrowTransitionAction::ResolveRelease { .. } => "m4_escrow_released",
        DataLayerM4EscrowTransitionAction::ResolveRefund { .. } => "m4_escrow_refunded",
        DataLayerM4EscrowTransitionAction::Expire { .. } => "m4_escrow_expired",
    }
}

fn compute_evidence_hash(
    sequence: u64,
    input: &DataLayerM4SettlementEvidenceInput,
    hash_chain_prev: &str,
) -> String {
    tagged_digest(
        format!(
            "m4-evidence|escrow:{}|seq:{sequence}|state:{}|receipt:{}|payload:{}|recorded:{}|prev:{}",
            input.escrow_id,
            input.escrow_state.as_marker(),
            input.settlement_receipt_hash,
            input.settlement_payload_hash,
            input.recorded_at_epoch_seconds,
            hash_chain_prev
        )
        .as_str(),
    )
}

fn tagged_digest(value: &str) -> String {
    format!(
        "{DATA_LAYER_M4_HASH_ALGORITHM}:{}",
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
        for (offset, byte) in value.as_bytes().iter().enumerate() {
            let mix = ((*byte as u64) << ((offset % 8) * 8))
                ^ ((offset as u64).wrapping_mul(0x100000001b3));
            acc ^= mix;
            acc = acc.rotate_left(((offset + index) % 63 + 1) as u32);
            acc = acc.wrapping_mul(0x100000001b3);
            acc ^= acc >> 29;
            acc = acc.wrapping_add(0x9e3779b97f4a7c15 ^ (index as u64));
        }
        output.push_str(format!("{acc:016x}").as_str());
    }
    output
}
