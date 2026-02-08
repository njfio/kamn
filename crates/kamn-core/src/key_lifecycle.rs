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
