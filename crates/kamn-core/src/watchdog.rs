use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogSeverity {
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchdogAlertKind {
    InvalidBlockParent {
        block_id: String,
        expected_parent: String,
        observed_parent: String,
    },
    CensorshipSignal {
        message_id: String,
        delivered_recipients: usize,
        expected_recipients: usize,
        observed_ratio_pct: u8,
    },
    QuorumAnomaly {
        block_id: String,
        observed_signatures: u16,
        min_required_signatures: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchdogAlert {
    pub severity: WatchdogSeverity,
    pub kind: WatchdogAlertKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchdogConfig {
    pub min_quorum_signatures: u16,
    pub min_delivery_ratio_pct: u8,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            min_quorum_signatures: 3,
            min_delivery_ratio_pct: 70,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchdogObservation {
    Block {
        block_id: String,
        state_hash: String,
        parent_state_hash: String,
        quorum_signatures: u16,
        expected_validator_count: u16,
    },
    GossipDelivery {
        message_id: String,
        target_recipients: usize,
        delivered_recipients: usize,
        expected_recipients: usize,
    },
}

impl WatchdogObservation {
    pub fn block(
        block_id: &str,
        state_hash: &str,
        parent_state_hash: &str,
        quorum_signatures: u16,
        expected_validator_count: u16,
    ) -> Self {
        Self::Block {
            block_id: block_id.to_owned(),
            state_hash: state_hash.to_owned(),
            parent_state_hash: parent_state_hash.to_owned(),
            quorum_signatures,
            expected_validator_count,
        }
    }

    pub fn gossip_delivery(
        message_id: &str,
        target_recipients: usize,
        delivered_recipients: usize,
        expected_recipients: usize,
    ) -> Self {
        Self::GossipDelivery {
            message_id: message_id.to_owned(),
            target_recipients,
            delivered_recipients,
            expected_recipients,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchdogSnapshot {
    pub total_observations: usize,
    pub total_alerts: usize,
    pub warning_alerts: usize,
    pub critical_alerts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchdogNode {
    config: WatchdogConfig,
    last_state_hash: Option<String>,
    total_observations: usize,
    warning_alerts: usize,
    critical_alerts: usize,
}

impl WatchdogNode {
    pub fn new(config: WatchdogConfig) -> Result<Self, WatchdogError> {
        validate_config(config)?;

        Ok(Self {
            config,
            last_state_hash: None,
            total_observations: 0,
            warning_alerts: 0,
            critical_alerts: 0,
        })
    }

    pub fn observe(
        &mut self,
        observation: WatchdogObservation,
    ) -> Result<Vec<WatchdogAlert>, WatchdogError> {
        let alerts = match observation {
            WatchdogObservation::Block {
                block_id,
                state_hash,
                parent_state_hash,
                quorum_signatures,
                expected_validator_count,
            } => self.observe_block(
                block_id,
                state_hash,
                parent_state_hash,
                quorum_signatures,
                expected_validator_count,
            )?,
            WatchdogObservation::GossipDelivery {
                message_id,
                target_recipients,
                delivered_recipients,
                expected_recipients,
            } => self.observe_gossip(
                message_id,
                target_recipients,
                delivered_recipients,
                expected_recipients,
            )?,
        };

        self.total_observations += 1;
        for alert in &alerts {
            match alert.severity {
                WatchdogSeverity::Warning => self.warning_alerts += 1,
                WatchdogSeverity::Critical => self.critical_alerts += 1,
            }
        }

        Ok(alerts)
    }

    pub fn snapshot(&self) -> WatchdogSnapshot {
        WatchdogSnapshot {
            total_observations: self.total_observations,
            total_alerts: self.warning_alerts + self.critical_alerts,
            warning_alerts: self.warning_alerts,
            critical_alerts: self.critical_alerts,
        }
    }

    fn observe_block(
        &mut self,
        block_id: String,
        state_hash: String,
        parent_state_hash: String,
        quorum_signatures: u16,
        expected_validator_count: u16,
    ) -> Result<Vec<WatchdogAlert>, WatchdogError> {
        require_non_empty("block_id", &block_id)?;
        require_non_empty("state_hash", &state_hash)?;
        require_non_empty("parent_state_hash", &parent_state_hash)?;
        if expected_validator_count == 0 {
            return Err(WatchdogError::InvalidObservation(
                "expected_validator_count must be greater than zero".to_owned(),
            ));
        }

        let mut alerts = Vec::new();

        if let Some(expected_parent) = &self.last_state_hash {
            if &parent_state_hash != expected_parent {
                alerts.push(WatchdogAlert {
                    severity: WatchdogSeverity::Critical,
                    kind: WatchdogAlertKind::InvalidBlockParent {
                        block_id: block_id.clone(),
                        expected_parent: expected_parent.clone(),
                        observed_parent: parent_state_hash,
                    },
                });
            }
        }

        if quorum_signatures < self.config.min_quorum_signatures {
            alerts.push(WatchdogAlert {
                severity: WatchdogSeverity::Critical,
                kind: WatchdogAlertKind::QuorumAnomaly {
                    block_id,
                    observed_signatures: quorum_signatures,
                    min_required_signatures: self.config.min_quorum_signatures,
                },
            });
        }

        self.last_state_hash = Some(state_hash);
        Ok(alerts)
    }

    fn observe_gossip(
        &self,
        message_id: String,
        target_recipients: usize,
        delivered_recipients: usize,
        expected_recipients: usize,
    ) -> Result<Vec<WatchdogAlert>, WatchdogError> {
        require_non_empty("message_id", &message_id)?;
        if expected_recipients == 0 {
            return Err(WatchdogError::InvalidObservation(
                "expected_recipients must be greater than zero".to_owned(),
            ));
        }
        if target_recipients == 0 {
            return Err(WatchdogError::InvalidObservation(
                "target_recipients must be greater than zero".to_owned(),
            ));
        }
        if delivered_recipients > expected_recipients {
            return Err(WatchdogError::InvalidObservation(
                "delivered_recipients cannot exceed expected_recipients".to_owned(),
            ));
        }

        // Direct single-recipient traffic should not be classified as censorship.
        if expected_recipients <= 1 {
            return Ok(Vec::new());
        }

        let observed_ratio_pct =
            ((delivered_recipients as f64 / expected_recipients as f64) * 100.0).floor() as u8;
        if observed_ratio_pct >= self.config.min_delivery_ratio_pct {
            return Ok(Vec::new());
        }

        Ok(vec![WatchdogAlert {
            severity: WatchdogSeverity::Warning,
            kind: WatchdogAlertKind::CensorshipSignal {
                message_id,
                delivered_recipients,
                expected_recipients,
                observed_ratio_pct,
            },
        }])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchdogError {
    InvalidConfig(String),
    InvalidObservation(String),
}

impl fmt::Display for WatchdogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => write!(f, "invalid config: {message}"),
            Self::InvalidObservation(message) => write!(f, "invalid observation: {message}"),
        }
    }
}

impl std::error::Error for WatchdogError {}

fn validate_config(config: WatchdogConfig) -> Result<(), WatchdogError> {
    if config.min_quorum_signatures == 0 {
        return Err(WatchdogError::InvalidConfig(
            "min_quorum_signatures must be greater than zero".to_owned(),
        ));
    }
    if config.min_delivery_ratio_pct == 0 || config.min_delivery_ratio_pct > 100 {
        return Err(WatchdogError::InvalidConfig(
            "min_delivery_ratio_pct must be between 1 and 100".to_owned(),
        ));
    }

    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<(), WatchdogError> {
    if value.trim().is_empty() {
        return Err(WatchdogError::InvalidObservation(format!(
            "{field} must not be empty"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        WatchdogConfig, WatchdogError, WatchdogNode, WatchdogObservation, WatchdogSeverity,
    };

    #[test]
    fn config_rejects_out_of_range_delivery_ratio() {
        assert_eq!(
            WatchdogNode::new(WatchdogConfig {
                min_quorum_signatures: 2,
                min_delivery_ratio_pct: 101,
            }),
            Err(WatchdogError::InvalidConfig(
                "min_delivery_ratio_pct must be between 1 and 100".to_owned(),
            ))
        );
    }

    #[test]
    fn gossip_rejects_delivered_above_expected() {
        let watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("valid config");
        let result = watchdog.observe_gossip("msg-1".to_owned(), 5, 6, 5);
        assert_eq!(
            result,
            Err(WatchdogError::InvalidObservation(
                "delivered_recipients cannot exceed expected_recipients".to_owned(),
            ))
        );
    }

    #[test]
    fn high_delivery_ratio_is_not_flagged() {
        let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("valid config");
        let alerts = watchdog
            .observe(WatchdogObservation::gossip_delivery("msg-1", 10, 8, 10))
            .expect("observation should be valid");
        assert!(alerts.is_empty());
        assert_eq!(watchdog.snapshot().warning_alerts, 0);
        assert_eq!(watchdog.snapshot().critical_alerts, 0);
    }

    #[test]
    fn low_quorum_is_critical() {
        let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("valid config");
        let alerts = watchdog
            .observe(WatchdogObservation::block(
                "block-1", "state-1", "state-0", 1, 5,
            ))
            .expect("observation should be valid");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, WatchdogSeverity::Critical);
    }
}
