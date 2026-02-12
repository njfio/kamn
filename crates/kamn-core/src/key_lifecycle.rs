/// Key lifecycle state for an agent signing key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyLifecycleState {
    /// Key is active and can sign traffic.
    Active,
    /// Rotation has started and a pending key is staged.
    Rotating,
    /// Key has been revoked and can no longer be used.
    Revoked,
}

/// Key lifecycle event emitted for audit and replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLifecycleEvent {
    /// Rotation started from one key id to a new key id.
    RotationInitiated {
        /// Monotonic lifecycle sequence number.
        sequence: u64,
        /// Previous active key id.
        from_key: String,
        /// Staged next key id.
        to_key: String,
    },
    /// Rotation was activated and pending key became active.
    RotationActivated {
        /// Monotonic lifecycle sequence number.
        sequence: u64,
        /// Newly active key id.
        active_key: String,
    },
    /// Active key was revoked.
    KeyRevoked {
        /// Monotonic lifecycle sequence number.
        sequence: u64,
        /// Revoked key id.
        key_id: String,
    },
}

/// Canonical audit record for a single lifecycle event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLifecycleAuditRecord {
    /// Monotonic lifecycle sequence number.
    pub sequence: u64,
    /// Canonical event kind.
    pub event_kind: String,
    /// Canonical event payload.
    pub event_payload: String,
    /// Previous record hash in audit chain.
    pub previous_hash: String,
    /// Record hash computed from canonical fields.
    pub record_hash: String,
}

/// Errors returned while validating lifecycle audit trails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLifecycleAuditError {
    /// Audit trail was empty.
    EmptyAuditTrail,
    /// Sequence numbers were not contiguous.
    SequenceGap {
        /// Expected sequence value.
        expected: u64,
        /// Observed sequence value.
        found: u64,
    },
    /// Record previous-hash link did not match chain state.
    BrokenHashChain {
        /// Sequence where chain link failed.
        sequence: u64,
    },
    /// Record hash did not match canonical recomputation.
    HashMismatch {
        /// Sequence where hash mismatch occurred.
        sequence: u64,
    },
}

impl std::fmt::Display for KeyLifecycleAuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyAuditTrail => write!(f, "audit trail must not be empty"),
            Self::SequenceGap { expected, found } => write!(
                f,
                "audit sequence gap detected: expected {expected}, found {found}"
            ),
            Self::BrokenHashChain { sequence } => {
                write!(f, "audit hash chain link mismatch at sequence {sequence}")
            }
            Self::HashMismatch { sequence } => {
                write!(f, "audit hash mismatch at sequence {sequence}")
            }
        }
    }
}

impl std::error::Error for KeyLifecycleAuditError {}

impl KeyLifecycleEvent {
    fn sequence(&self) -> u64 {
        match self {
            Self::RotationInitiated { sequence, .. }
            | Self::RotationActivated { sequence, .. }
            | Self::KeyRevoked { sequence, .. } => *sequence,
        }
    }

    fn canonical_kind(&self) -> &'static str {
        match self {
            Self::RotationInitiated { .. } => "rotation_initiated",
            Self::RotationActivated { .. } => "rotation_activated",
            Self::KeyRevoked { .. } => "key_revoked",
        }
    }

    fn canonical_payload(&self) -> String {
        match self {
            Self::RotationInitiated {
                from_key, to_key, ..
            } => {
                format!("from_key={from_key};to_key={to_key}")
            }
            Self::RotationActivated { active_key, .. } => {
                format!("active_key={active_key}")
            }
            Self::KeyRevoked { key_id, .. } => {
                format!("key_id={key_id}")
            }
        }
    }
}

/// Key lifecycle state machine with deterministic audit-event emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLifecycle {
    state: KeyLifecycleState,
    active_key_id: String,
    pending_key_id: Option<String>,
    sequence: u64,
    events: Vec<KeyLifecycleEvent>,
}

impl KeyLifecycle {
    /// Creates lifecycle with an initial active key id.
    pub fn new(active_key_id: &str) -> Result<Self, KeyLifecycleError> {
        if active_key_id.trim().is_empty() {
            return Err(KeyLifecycleError::EmptyKeyId);
        }
        Ok(Self {
            state: KeyLifecycleState::Active,
            active_key_id: active_key_id.to_owned(),
            pending_key_id: None,
            sequence: 0,
            events: Vec::new(),
        })
    }

    /// Returns current lifecycle state.
    pub fn state(&self) -> KeyLifecycleState {
        self.state
    }

    /// Returns current active key id.
    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }

    /// Returns pending key id when rotation is in progress.
    pub fn pending_key_id(&self) -> Option<&str> {
        self.pending_key_id.as_deref()
    }

    /// Returns immutable lifecycle event history.
    pub fn events(&self) -> &[KeyLifecycleEvent] {
        &self.events
    }

    /// Builds canonical audit records from lifecycle events.
    pub fn audit_records(&self) -> Vec<KeyLifecycleAuditRecord> {
        let mut records = Vec::with_capacity(self.events.len());
        let mut previous_hash = AUDIT_CHAIN_GENESIS.to_owned();

        for event in &self.events {
            let sequence = event.sequence();
            let event_kind = event.canonical_kind().to_owned();
            let event_payload = event.canonical_payload();
            let record_hash =
                compute_audit_hash(sequence, &event_kind, &event_payload, &previous_hash);

            records.push(KeyLifecycleAuditRecord {
                sequence,
                event_kind,
                event_payload,
                previous_hash: previous_hash.clone(),
                record_hash: record_hash.clone(),
            });

            previous_hash = record_hash;
        }

        records
    }

    /// Verifies audit trail generated from in-memory lifecycle events.
    pub fn verify_audit_trail(&self) -> Result<(), KeyLifecycleAuditError> {
        Self::verify_audit_records(&self.audit_records())
    }

    /// Verifies provided audit records for sequence continuity and hash-chain integrity.
    pub fn verify_audit_records(
        records: &[KeyLifecycleAuditRecord],
    ) -> Result<(), KeyLifecycleAuditError> {
        if records.is_empty() {
            return Err(KeyLifecycleAuditError::EmptyAuditTrail);
        }

        let mut expected_sequence = 1;
        let mut previous_hash = AUDIT_CHAIN_GENESIS.to_owned();

        for record in records {
            if record.sequence != expected_sequence {
                return Err(KeyLifecycleAuditError::SequenceGap {
                    expected: expected_sequence,
                    found: record.sequence,
                });
            }

            if record.previous_hash != previous_hash {
                return Err(KeyLifecycleAuditError::BrokenHashChain {
                    sequence: record.sequence,
                });
            }

            let expected_hash = compute_audit_hash(
                record.sequence,
                &record.event_kind,
                &record.event_payload,
                &record.previous_hash,
            );
            if record.record_hash != expected_hash {
                return Err(KeyLifecycleAuditError::HashMismatch {
                    sequence: record.sequence,
                });
            }

            previous_hash = record.record_hash.clone();
            expected_sequence += 1;
        }

        Ok(())
    }

    /// Initiates rotation from active key to `next_key_id`.
    pub fn initiate_rotation(&mut self, next_key_id: &str) -> Result<(), KeyLifecycleError> {
        if next_key_id.trim().is_empty() {
            return Err(KeyLifecycleError::EmptyKeyId);
        }
        if self.state != KeyLifecycleState::Active {
            return Err(KeyLifecycleError::InvalidTransition {
                from: self.state,
                action: "initiate_rotation",
            });
        }
        if next_key_id == self.active_key_id {
            return Err(KeyLifecycleError::RotationKeyUnchanged);
        }

        self.sequence += 1;
        self.events.push(KeyLifecycleEvent::RotationInitiated {
            sequence: self.sequence,
            from_key: self.active_key_id.clone(),
            to_key: next_key_id.to_owned(),
        });
        self.pending_key_id = Some(next_key_id.to_owned());
        self.state = KeyLifecycleState::Rotating;
        Ok(())
    }

    /// Activates currently pending key and returns lifecycle to active state.
    pub fn activate_rotation(&mut self) -> Result<(), KeyLifecycleError> {
        if self.state != KeyLifecycleState::Rotating {
            return Err(KeyLifecycleError::InvalidTransition {
                from: self.state,
                action: "activate_rotation",
            });
        }
        let next_key = match self.pending_key_id.take() {
            Some(value) => value,
            None => {
                return Err(KeyLifecycleError::InvalidTransition {
                    from: self.state,
                    action: "activate_rotation",
                });
            }
        };

        self.active_key_id = next_key.clone();
        self.state = KeyLifecycleState::Active;
        self.sequence += 1;
        self.events.push(KeyLifecycleEvent::RotationActivated {
            sequence: self.sequence,
            active_key: next_key,
        });
        Ok(())
    }

    /// Revokes the active key and transitions lifecycle to revoked state.
    pub fn revoke(&mut self) -> Result<(), KeyLifecycleError> {
        match self.state {
            KeyLifecycleState::Active | KeyLifecycleState::Rotating => {}
            KeyLifecycleState::Revoked => {
                return Err(KeyLifecycleError::InvalidTransition {
                    from: self.state,
                    action: "revoke",
                });
            }
        }

        self.pending_key_id = None;
        self.state = KeyLifecycleState::Revoked;
        self.sequence += 1;
        self.events.push(KeyLifecycleEvent::KeyRevoked {
            sequence: self.sequence,
            key_id: self.active_key_id.clone(),
        });
        Ok(())
    }
}

const AUDIT_CHAIN_GENESIS: &str = "GENESIS";

fn compute_audit_hash(
    sequence: u64,
    event_kind: &str,
    event_payload: &str,
    previous_hash: &str,
) -> String {
    let canonical_payload = format!("{sequence}|{event_kind}|{event_payload}|{previous_hash}");

    // First slice uses a deterministic non-cryptographic hash; this can be replaced by SHA-256 later.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical_payload.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3_u64);
    }

    format!("{hash:016x}")
}

/// Errors emitted by key lifecycle transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLifecycleError {
    /// Key id input was empty.
    EmptyKeyId,
    /// Rotation target matched active key id.
    RotationKeyUnchanged,
    /// Transition/action is invalid for current lifecycle state.
    InvalidTransition {
        /// Current lifecycle state.
        from: KeyLifecycleState,
        /// Requested transition action.
        action: &'static str,
    },
}

impl std::fmt::Display for KeyLifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyKeyId => write!(f, "key id must not be empty"),
            Self::RotationKeyUnchanged => {
                write!(f, "rotation target key must differ from active key")
            }
            Self::InvalidTransition { from, action } => {
                write!(
                    f,
                    "invalid key lifecycle transition from {from:?} via {action}"
                )
            }
        }
    }
}

impl std::error::Error for KeyLifecycleError {}

#[cfg(test)]
mod tests {
    use super::{KeyLifecycle, KeyLifecycleError, KeyLifecycleState};

    #[test]
    fn new_rejects_empty_key() {
        assert_eq!(KeyLifecycle::new(""), Err(KeyLifecycleError::EmptyKeyId));
    }

    #[test]
    fn revoke_from_rotating_is_allowed() {
        let mut lifecycle = match KeyLifecycle::new("key_v1") {
            Ok(value) => value,
            Err(error) => panic!("init failed: {error}"),
        };
        if let Err(error) = lifecycle.initiate_rotation("key_v2") {
            panic!("rotation init failed: {error}");
        }
        if let Err(error) = lifecycle.revoke() {
            panic!("revoke failed: {error}");
        }
        assert_eq!(lifecycle.state(), KeyLifecycleState::Revoked);
    }
}
