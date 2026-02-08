use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AntiSpamConfig {
    pub max_messages_per_window: usize,
    pub window_seconds: u64,
    pub minimum_sybil_deposit: u64,
    pub suspension_violation_threshold: u32,
    pub suspension_seconds: u64,
}

impl Default for AntiSpamConfig {
    fn default() -> Self {
        Self {
            max_messages_per_window: 3,
            window_seconds: 5,
            minimum_sybil_deposit: 10,
            suspension_violation_threshold: 2,
            suspension_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AntiSpamDecision {
    Accepted,
    Rejected(AntiSpamRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AntiSpamRejection {
    InsufficientDeposit {
        required: u64,
        provided: u64,
    },
    RateLimitExceeded {
        limit: usize,
        observed: usize,
        window_seconds: u64,
    },
    SenderSuspended {
        until_unix: u64,
    },
    DuplicateMessageId(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AntiSpamTelemetry {
    pub total_processed: u64,
    pub accepted: u64,
    pub rejected_insufficient_deposit: u64,
    pub rejected_rate_limit: u64,
    pub rejected_suspended: u64,
    pub rejected_duplicate_message: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AntiSpamError {
    InvalidConfig(String),
    InvalidInput(String),
}

impl fmt::Display for AntiSpamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => write!(f, "invalid config: {message}"),
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
        }
    }
}

impl std::error::Error for AntiSpamError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct SenderState {
    recent_timestamps: VecDeque<u64>,
    consecutive_rate_violations: u32,
    suspended_until_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiSpamEngine {
    config: AntiSpamConfig,
    deposits: HashMap<String, u64>,
    seen_message_ids: HashSet<String>,
    sender_state: HashMap<String, SenderState>,
    telemetry: AntiSpamTelemetry,
}

impl AntiSpamEngine {
    pub fn new(config: AntiSpamConfig) -> Result<Self, AntiSpamError> {
        validate_config(config)?;

        Ok(Self {
            config,
            deposits: HashMap::new(),
            seen_message_ids: HashSet::new(),
            sender_state: HashMap::new(),
            telemetry: AntiSpamTelemetry::default(),
        })
    }

    pub fn set_deposit(&mut self, sender_did: &str, deposit: u64) -> Result<(), AntiSpamError> {
        validate_sender_did(sender_did)?;
        self.deposits.insert(sender_did.to_owned(), deposit);
        Ok(())
    }

    pub fn evaluate(
        &mut self,
        sender_did: &str,
        message_id: &str,
        now_unix: u64,
    ) -> Result<AntiSpamDecision, AntiSpamError> {
        validate_sender_did(sender_did)?;
        require_non_empty("message_id", message_id)?;

        self.telemetry.total_processed += 1;

        if !self.seen_message_ids.insert(message_id.to_owned()) {
            self.telemetry.rejected_duplicate_message += 1;
            return Ok(AntiSpamDecision::Rejected(
                AntiSpamRejection::DuplicateMessageId(message_id.to_owned()),
            ));
        }

        let deposit = *self.deposits.get(sender_did).unwrap_or(&0);
        if deposit < self.config.minimum_sybil_deposit {
            self.telemetry.rejected_insufficient_deposit += 1;
            return Ok(AntiSpamDecision::Rejected(
                AntiSpamRejection::InsufficientDeposit {
                    required: self.config.minimum_sybil_deposit,
                    provided: deposit,
                },
            ));
        }

        let state = self.sender_state.entry(sender_did.to_owned()).or_default();

        if let Some(until_unix) = state.suspended_until_unix {
            if now_unix < until_unix {
                self.telemetry.rejected_suspended += 1;
                return Ok(AntiSpamDecision::Rejected(
                    AntiSpamRejection::SenderSuspended { until_unix },
                ));
            }
        }

        while let Some(oldest) = state.recent_timestamps.front() {
            if now_unix.saturating_sub(*oldest) < self.config.window_seconds {
                break;
            }
            state.recent_timestamps.pop_front();
        }

        let observed = state.recent_timestamps.len();
        if observed >= self.config.max_messages_per_window {
            state.consecutive_rate_violations = state.consecutive_rate_violations.saturating_add(1);
            if state.consecutive_rate_violations >= self.config.suspension_violation_threshold {
                state.suspended_until_unix =
                    Some(now_unix.saturating_add(self.config.suspension_seconds));
            }
            self.telemetry.rejected_rate_limit += 1;
            return Ok(AntiSpamDecision::Rejected(
                AntiSpamRejection::RateLimitExceeded {
                    limit: self.config.max_messages_per_window,
                    observed,
                    window_seconds: self.config.window_seconds,
                },
            ));
        }

        state.recent_timestamps.push_back(now_unix);
        state.consecutive_rate_violations = 0;

        self.telemetry.accepted += 1;
        Ok(AntiSpamDecision::Accepted)
    }

    pub fn telemetry(&self) -> AntiSpamTelemetry {
        self.telemetry
    }
}

fn validate_config(config: AntiSpamConfig) -> Result<(), AntiSpamError> {
    if config.max_messages_per_window == 0 {
        return Err(AntiSpamError::InvalidConfig(
            "max_messages_per_window must be greater than zero".to_owned(),
        ));
    }
    if config.window_seconds == 0 {
        return Err(AntiSpamError::InvalidConfig(
            "window_seconds must be greater than zero".to_owned(),
        ));
    }
    if config.suspension_violation_threshold == 0 {
        return Err(AntiSpamError::InvalidConfig(
            "suspension_violation_threshold must be greater than zero".to_owned(),
        ));
    }
    if config.suspension_seconds == 0 {
        return Err(AntiSpamError::InvalidConfig(
            "suspension_seconds must be greater than zero".to_owned(),
        ));
    }

    Ok(())
}

fn validate_sender_did(value: &str) -> Result<(), AntiSpamError> {
    require_non_empty("sender_did", value)?;
    if !value.starts_with("kamn:did:agent:") {
        return Err(AntiSpamError::InvalidInput(
            "sender_did must use kamn:did:agent:* format".to_owned(),
        ));
    }

    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<(), AntiSpamError> {
    if value.trim().is_empty() {
        return Err(AntiSpamError::InvalidInput(format!(
            "{field} must not be empty"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AntiSpamConfig, AntiSpamDecision, AntiSpamEngine, AntiSpamError};

    #[test]
    fn invalid_sender_did_is_rejected() {
        let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("valid config");
        let result = engine.set_deposit("did:example:abc", 10);
        assert_eq!(
            result,
            Err(AntiSpamError::InvalidInput(
                "sender_did must use kamn:did:agent:* format".to_owned(),
            ))
        );
    }

    #[test]
    fn default_engine_accepts_first_message_with_sufficient_deposit() {
        let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("valid config");
        engine
            .set_deposit("kamn:did:agent:sender-1", 20)
            .expect("deposit should be accepted");

        let decision = engine
            .evaluate("kamn:did:agent:sender-1", "msg-1", 1)
            .expect("evaluation should succeed");
        assert_eq!(decision, AntiSpamDecision::Accepted);
    }

    #[test]
    fn unknown_sender_without_deposit_is_rejected() {
        let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("valid config");
        let decision = engine
            .evaluate("kamn:did:agent:sender-unknown", "msg-1", 1)
            .expect("evaluation should succeed");
        assert!(matches!(
            decision,
            AntiSpamDecision::Rejected(super::AntiSpamRejection::InsufficientDeposit { .. })
        ));
    }
}
