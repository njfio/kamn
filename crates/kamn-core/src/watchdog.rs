//! Runtime watchdog anomaly classification and alerting contracts.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Alert severity level emitted by watchdog analysis.
pub enum WatchdogSeverity {
    /// Warning-level anomaly.
    Warning,
    /// Critical anomaly.
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Anomaly taxonomy produced by watchdog observations.
pub enum WatchdogAlertKind {
    /// Block parent hash mismatch anomaly.
    InvalidBlockParent {
        /// Block identifier.
        block_id: String,
        /// Expected parent hash.
        expected_parent: String,
        /// Observed parent hash.
        observed_parent: String,
    },
    /// Potential censorship signal from delivery ratio degradation.
    CensorshipSignal {
        /// Message identifier.
        message_id: String,
        /// Number of delivered recipients.
        delivered_recipients: usize,
        /// Number of expected recipients.
        expected_recipients: usize,
        /// Observed delivery ratio percentage.
        observed_ratio_pct: u8,
    },
    /// Quorum-signature anomaly for a block.
    QuorumAnomaly {
        /// Block identifier.
        block_id: String,
        /// Observed signature count.
        observed_signatures: u16,
        /// Minimum required signature count.
        min_required_signatures: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Alert record emitted by watchdog analysis.
pub struct WatchdogAlert {
    /// Alert severity.
    pub severity: WatchdogSeverity,
    /// Alert kind payload.
    pub kind: WatchdogAlertKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Runtime watchdog policy configuration.
pub struct WatchdogConfig {
    /// Minimum signatures required for block quorum.
    pub min_quorum_signatures: u16,
    /// Minimum healthy delivery ratio percentage.
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
/// Observation payload ingested by watchdog analysis.
pub enum WatchdogObservation {
    /// Block-level observation.
    Block {
        /// Block identifier.
        block_id: String,
        /// Current block state hash.
        state_hash: String,
        /// Parent block state hash.
        parent_state_hash: String,
        /// Observed quorum signatures.
        quorum_signatures: u16,
        /// Expected validator count for this block.
        expected_validator_count: u16,
    },
    /// Gossip delivery observation.
    GossipDelivery {
        /// Message identifier.
        message_id: String,
        /// Target recipients count.
        target_recipients: usize,
        /// Delivered recipients count.
        delivered_recipients: usize,
        /// Expected recipients count.
        expected_recipients: usize,
    },
}

impl WatchdogObservation {
    /// Constructs a block observation payload.
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

    /// Constructs a gossip delivery observation payload.
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
/// Aggregate watchdog counters snapshot.
pub struct WatchdogSnapshot {
    /// Total observations processed.
    pub total_observations: usize,
    /// Total emitted alerts.
    pub total_alerts: usize,
    /// Total warning alerts.
    pub warning_alerts: usize,
    /// Total critical alerts.
    pub critical_alerts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Stateful watchdog engine for runtime anomaly classification.
pub struct WatchdogNode {
    config: WatchdogConfig,
    last_state_hash: Option<String>,
    total_observations: usize,
    warning_alerts: usize,
    critical_alerts: usize,
}

impl WatchdogNode {
    /// Creates a watchdog node from validated config.
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

    /// Processes an observation and returns emitted alerts.
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

    /// Returns aggregate counters for processed observations and alerts.
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
/// Error taxonomy for watchdog configuration and observation validation.
pub enum WatchdogError {
    /// Configuration is invalid.
    InvalidConfig(String),
    /// Observation payload is invalid.
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
