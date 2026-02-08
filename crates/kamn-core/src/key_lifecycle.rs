#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyLifecycleState {
    Active,
    Rotating,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLifecycleEvent {
    RotationInitiated {
        sequence: u64,
        from_key: String,
        to_key: String,
    },
    RotationActivated {
        sequence: u64,
        active_key: String,
    },
    KeyRevoked {
        sequence: u64,
        key_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLifecycleAuditRecord {
    pub sequence: u64,
    pub event_kind: String,
    pub event_payload: String,
    pub previous_hash: String,
    pub record_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLifecycleAuditError {
    EmptyAuditTrail,
    SequenceGap { expected: u64, found: u64 },
    BrokenHashChain { sequence: u64 },
    HashMismatch { sequence: u64 },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLifecycle {
    state: KeyLifecycleState,
    active_key_id: String,
    pending_key_id: Option<String>,
    sequence: u64,
    events: Vec<KeyLifecycleEvent>,
}

impl KeyLifecycle {
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

    pub fn state(&self) -> KeyLifecycleState {
        self.state
    }

    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }

    pub fn pending_key_id(&self) -> Option<&str> {
        self.pending_key_id.as_deref()
    }

    pub fn events(&self) -> &[KeyLifecycleEvent] {
        &self.events
    }

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

    pub fn verify_audit_trail(&self) -> Result<(), KeyLifecycleAuditError> {
        Self::verify_audit_records(&self.audit_records())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyLifecycleError {
    EmptyKeyId,
    RotationKeyUnchanged,
    InvalidTransition {
        from: KeyLifecycleState,
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
