use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Watchdog anomaly kind.
pub enum WatchdogAnomalyKind {
    /// Nominal.
    Nominal,
    /// Liveness degradation.
    LivenessDegradation,
    /// Censorship signal.
    CensorshipSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Watchdog anomaly severity.
pub enum WatchdogAnomalySeverity {
    /// Info.
    Info,
    /// Warning.
    Warning,
    /// Critical.
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Watchdog anomaly watch input.
pub struct WatchdogAnomalyWatchInput {
    sample_id: String,
    expected_deliveries: u32,
    delivered_deliveries: u32,
    active_peers: u32,
    healthy_peers: u32,
    sample_window_secs: u64,
    targeted_peer_count: u32,
}

impl WatchdogAnomalyWatchInput {
    #[allow(clippy::too_many_arguments)]
    /// Handles new.
    pub fn new(
        sample_id: &str,
        expected_deliveries: u32,
        delivered_deliveries: u32,
        active_peers: u32,
        healthy_peers: u32,
        sample_window_secs: u64,
        targeted_peer_count: u32,
    ) -> Result<Self, WatchdogAnomalyError> {
        if sample_id.trim().is_empty() {
            return Err(WatchdogAnomalyError::InvalidSampleId);
        }
        if expected_deliveries == 0 {
            return Err(WatchdogAnomalyError::InvalidExpectedDeliveries {
                expected_deliveries,
            });
        }
        if delivered_deliveries > expected_deliveries {
            return Err(WatchdogAnomalyError::InvalidSampleCounts {
                expected_deliveries,
                delivered_deliveries,
            });
        }
        if active_peers == 0 || healthy_peers > active_peers {
            return Err(WatchdogAnomalyError::InvalidPeerCounts {
                active_peers,
                healthy_peers,
            });
        }
        if sample_window_secs == 0 {
            return Err(WatchdogAnomalyError::InvalidSampleWindow { sample_window_secs });
        }

        Ok(Self {
            sample_id: sample_id.to_owned(),
            expected_deliveries,
            delivered_deliveries,
            active_peers,
            healthy_peers,
            sample_window_secs,
            targeted_peer_count,
        })
    }

    fn delivery_ratio_per_mille(&self) -> u16 {
        ((self.delivered_deliveries as u64) * 1000 / self.expected_deliveries as u64) as u16
    }

    fn liveness_ratio_per_mille(&self) -> u16 {
        ((self.healthy_peers as u64) * 1000 / self.active_peers as u64) as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Watchdog anomaly report.
pub struct WatchdogAnomalyReport {
    /// Sample id.
    pub sample_id: String,
    /// Kind.
    pub kind: WatchdogAnomalyKind,
    /// Severity.
    pub severity: WatchdogAnomalySeverity,
    /// Delivery ratio per mille.
    pub delivery_ratio_per_mille: u16,
    /// Liveness ratio per mille.
    pub liveness_ratio_per_mille: u16,
    /// Targeted peer count.
    pub targeted_peer_count: u32,
    /// Sample window secs.
    pub sample_window_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Watchdog anomaly error.
pub enum WatchdogAnomalyError {
    /// Invalid sample id.
    InvalidSampleId,
    /// Invalid expected deliveries.
    InvalidExpectedDeliveries {
        /// Expected deliveries.
        expected_deliveries: u32,
    },
    /// Invalid sample counts.
    InvalidSampleCounts {
        /// Expected deliveries.
        expected_deliveries: u32,
        /// Delivered deliveries.
        delivered_deliveries: u32,
    },
    /// Invalid peer counts.
    InvalidPeerCounts {
        /// Active peers.
        active_peers: u32,
        /// Healthy peers.
        healthy_peers: u32,
    },
    /// Invalid sample window.
    InvalidSampleWindow {
        /// Sample window secs.
        sample_window_secs: u64,
    },
}

impl Display for WatchdogAnomalyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleId => write!(f, "watchdog anomaly sample id cannot be empty"),
            Self::InvalidExpectedDeliveries {
                expected_deliveries,
            } => write!(
                f,
                "watchdog anomaly expected deliveries must be positive, found {expected_deliveries}"
            ),
            Self::InvalidSampleCounts {
                expected_deliveries,
                delivered_deliveries,
            } => write!(
                f,
                "watchdog anomaly delivered deliveries {delivered_deliveries} exceed expected {expected_deliveries}"
            ),
            Self::InvalidPeerCounts {
                active_peers,
                healthy_peers,
            } => write!(
                f,
                "watchdog anomaly peer counts are invalid: active {active_peers}, healthy {healthy_peers}"
            ),
            Self::InvalidSampleWindow { sample_window_secs } => write!(
                f,
                "watchdog anomaly sample window must be positive, found {sample_window_secs}"
            ),
        }
    }
}

impl Error for WatchdogAnomalyError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Watchdog anomaly evaluator.
pub struct WatchdogAnomalyEvaluator;

impl WatchdogAnomalyEvaluator {
    /// Handles evaluate.
    pub fn evaluate(
        &self,
        input: WatchdogAnomalyWatchInput,
    ) -> Result<WatchdogAnomalyReport, WatchdogAnomalyError> {
        let delivery_ratio_per_mille = input.delivery_ratio_per_mille();
        let liveness_ratio_per_mille = input.liveness_ratio_per_mille();

        let (kind, severity) = if input.targeted_peer_count >= 2 && delivery_ratio_per_mille <= 500
        {
            (
                WatchdogAnomalyKind::CensorshipSignal,
                WatchdogAnomalySeverity::Critical,
            )
        } else if input.targeted_peer_count >= 2 && delivery_ratio_per_mille <= 850 {
            (
                WatchdogAnomalyKind::CensorshipSignal,
                WatchdogAnomalySeverity::Warning,
            )
        } else if liveness_ratio_per_mille <= 500 {
            (
                WatchdogAnomalyKind::LivenessDegradation,
                WatchdogAnomalySeverity::Critical,
            )
        } else if liveness_ratio_per_mille < 1000 {
            (
                WatchdogAnomalyKind::LivenessDegradation,
                WatchdogAnomalySeverity::Warning,
            )
        } else {
            (WatchdogAnomalyKind::Nominal, WatchdogAnomalySeverity::Info)
        };

        Ok(WatchdogAnomalyReport {
            sample_id: input.sample_id,
            kind,
            severity,
            delivery_ratio_per_mille,
            liveness_ratio_per_mille,
            targeted_peer_count: input.targeted_peer_count,
            sample_window_secs: input.sample_window_secs,
        })
    }
}

/// Handles evaluate daemon watchdog anomaly.
pub fn evaluate_daemon_watchdog_anomaly(
    evaluator: &WatchdogAnomalyEvaluator,
    input: WatchdogAnomalyWatchInput,
) -> Result<WatchdogAnomalyReport, WatchdogAnomalyError> {
    evaluator.evaluate(input)
}
