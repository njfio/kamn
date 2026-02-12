//! Observability sampling and SLO evaluation contracts for runtime health.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Metric dimensions tracked by the observability monitor.
pub enum ObservabilityMetric {
    /// Median latency in milliseconds.
    LatencyP50,
    /// Tail latency (p99) in milliseconds.
    LatencyP99,
    /// Throughput in operations per second.
    Throughput,
    /// Error-rate percentage.
    ErrorRate,
    /// Availability percentage.
    Availability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Alert severity level.
pub enum ObservabilitySeverity {
    /// Warning-level threshold breach.
    Warning,
    /// Critical threshold breach.
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Aggregate health state after alert classification.
pub enum ObservabilityHealth {
    /// No alert thresholds were breached.
    Healthy,
    /// Warning-level alerts exist but no critical alerts.
    Degraded,
    /// At least one critical alert exists.
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
/// Single observability sample captured at a point in time.
pub struct ObservabilitySample {
    /// Observed p50 latency (milliseconds).
    pub latency_p50_ms: u64,
    /// Observed p99 latency (milliseconds).
    pub latency_p99_ms: u64,
    /// Observed throughput (operations per second).
    pub throughput_tps: u64,
    /// Observed error-rate percentage.
    pub error_rate_pct: f64,
    /// Observed availability percentage.
    pub availability_pct: f64,
    /// Sample timestamp (epoch seconds).
    pub timestamp_epoch_s: u64,
}

#[derive(Debug, Clone, PartialEq)]
/// SLO threshold profile used for observability evaluation.
pub struct ObservabilitySloProfile {
    /// Maximum allowed p50 latency.
    pub max_latency_p50_ms: u64,
    /// Maximum allowed p99 latency.
    pub max_latency_p99_ms: u64,
    /// Minimum required throughput.
    pub min_throughput_tps: u64,
    /// Maximum allowed error-rate percentage.
    pub max_error_rate_pct: f64,
    /// Minimum required availability percentage.
    pub min_availability_pct: f64,
}

impl ObservabilitySloProfile {
    /// Returns the baseline SLO threshold profile.
    pub fn baseline() -> Self {
        Self {
            max_latency_p50_ms: 100,
            max_latency_p99_ms: 350,
            min_throughput_tps: 1_700,
            max_error_rate_pct: 1.0,
            min_availability_pct: 99.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Alert entry generated from a metric threshold breach.
pub struct ObservabilityAlert {
    /// Metric that breached its threshold.
    pub metric: ObservabilityMetric,
    /// Classified severity for the breach.
    pub severity: ObservabilitySeverity,
    /// Observed metric value.
    pub observed: f64,
    /// Threshold value used for comparison.
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
/// Evaluation output for a single sample.
pub struct ObservabilityReport {
    /// Evaluated sample.
    pub sample: ObservabilitySample,
    /// Overall health derived from generated alerts.
    pub overall_health: ObservabilityHealth,
    /// Alerts produced for this sample.
    pub alerts: Vec<ObservabilityAlert>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Rolling health summary over monitor history.
pub struct ObservabilitySnapshot {
    /// Total number of evaluated samples.
    pub total_samples: usize,
    /// Count of healthy samples.
    pub healthy_samples: usize,
    /// Count of degraded samples.
    pub degraded_samples: usize,
    /// Count of critical samples.
    pub critical_samples: usize,
    /// Health state of the latest sample.
    pub latest_health: ObservabilityHealth,
}

#[derive(Debug, Clone, PartialEq)]
/// In-memory observability monitor with SLO-based alerting.
pub struct ObservabilityMonitor {
    profile: ObservabilitySloProfile,
    history: Vec<ObservabilityReport>,
}

impl ObservabilityMonitor {
    /// Creates an observability monitor for `profile`.
    pub fn new(profile: ObservabilitySloProfile) -> Self {
        Self {
            profile,
            history: Vec::new(),
        }
    }

    /// Evaluates a sample against SLO thresholds and stores the report.
    pub fn evaluate(
        &mut self,
        sample: ObservabilitySample,
    ) -> Result<ObservabilityReport, ObservabilityError> {
        validate_sample(&sample)?;
        let mut alerts = Vec::new();

        if sample.latency_p50_ms > self.profile.max_latency_p50_ms {
            alerts.push(ObservabilityAlert {
                metric: ObservabilityMetric::LatencyP50,
                severity: ObservabilitySeverity::Warning,
                observed: sample.latency_p50_ms as f64,
                threshold: self.profile.max_latency_p50_ms as f64,
            });
        }

        if sample.latency_p99_ms > self.profile.max_latency_p99_ms {
            alerts.push(ObservabilityAlert {
                metric: ObservabilityMetric::LatencyP99,
                severity: ObservabilitySeverity::Critical,
                observed: sample.latency_p99_ms as f64,
                threshold: self.profile.max_latency_p99_ms as f64,
            });
        }

        if sample.throughput_tps < self.profile.min_throughput_tps {
            alerts.push(ObservabilityAlert {
                metric: ObservabilityMetric::Throughput,
                severity: ObservabilitySeverity::Warning,
                observed: sample.throughput_tps as f64,
                threshold: self.profile.min_throughput_tps as f64,
            });
        }

        let error_severity = if sample.error_rate_pct > self.profile.max_error_rate_pct {
            if sample.error_rate_pct > self.profile.max_error_rate_pct * 2.0 {
                ObservabilitySeverity::Critical
            } else {
                ObservabilitySeverity::Warning
            }
        } else {
            ObservabilitySeverity::Warning
        };
        if sample.error_rate_pct > self.profile.max_error_rate_pct {
            alerts.push(ObservabilityAlert {
                metric: ObservabilityMetric::ErrorRate,
                severity: error_severity,
                observed: sample.error_rate_pct,
                threshold: self.profile.max_error_rate_pct,
            });
        }

        if sample.availability_pct < self.profile.min_availability_pct {
            alerts.push(ObservabilityAlert {
                metric: ObservabilityMetric::Availability,
                severity: ObservabilitySeverity::Critical,
                observed: sample.availability_pct,
                threshold: self.profile.min_availability_pct,
            });
        }

        let overall_health = if alerts
            .iter()
            .any(|alert| alert.severity == ObservabilitySeverity::Critical)
        {
            ObservabilityHealth::Critical
        } else if alerts.is_empty() {
            ObservabilityHealth::Healthy
        } else {
            ObservabilityHealth::Degraded
        };

        let report = ObservabilityReport {
            sample,
            overall_health,
            alerts,
        };
        self.history.push(report.clone());
        Ok(report)
    }

    /// Produces a summary snapshot across all evaluated samples.
    pub fn snapshot(&self) -> ObservabilitySnapshot {
        let total_samples = self.history.len();
        let healthy_samples = self
            .history
            .iter()
            .filter(|report| report.overall_health == ObservabilityHealth::Healthy)
            .count();
        let degraded_samples = self
            .history
            .iter()
            .filter(|report| report.overall_health == ObservabilityHealth::Degraded)
            .count();
        let critical_samples = self
            .history
            .iter()
            .filter(|report| report.overall_health == ObservabilityHealth::Critical)
            .count();
        let latest_health = self
            .history
            .last()
            .map(|report| report.overall_health)
            .unwrap_or(ObservabilityHealth::Healthy);

        ObservabilitySnapshot {
            total_samples,
            healthy_samples,
            degraded_samples,
            critical_samples,
            latest_health,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Error taxonomy for observability sample validation failures.
pub enum ObservabilityError {
    /// Percentage field is outside `0..=100`.
    InvalidPercentage {
        /// Name of the invalid field.
        field: &'static str,
        /// Invalid percentage value.
        value: f64,
    },
    /// p99 latency is lower than p50 latency.
    InvalidLatencyOrder {
        /// Observed p50 latency.
        p50: u64,
        /// Observed p99 latency.
        p99: u64,
    },
    /// Throughput field is zero.
    ZeroThroughput,
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPercentage { field, value } => {
                write!(f, "{field} must be between 0 and 100, found {value}")
            }
            Self::InvalidLatencyOrder { p50, p99 } => {
                write!(
                    f,
                    "latency ordering invalid: p99 ({p99}) must be >= p50 ({p50})"
                )
            }
            Self::ZeroThroughput => write!(f, "throughput_tps must be greater than zero"),
        }
    }
}

impl std::error::Error for ObservabilityError {}

fn validate_sample(sample: &ObservabilitySample) -> Result<(), ObservabilityError> {
    if !(0.0..=100.0).contains(&sample.error_rate_pct) {
        return Err(ObservabilityError::InvalidPercentage {
            field: "error_rate_pct",
            value: sample.error_rate_pct,
        });
    }
    if !(0.0..=100.0).contains(&sample.availability_pct) {
        return Err(ObservabilityError::InvalidPercentage {
            field: "availability_pct",
            value: sample.availability_pct,
        });
    }
    if sample.latency_p99_ms < sample.latency_p50_ms {
        return Err(ObservabilityError::InvalidLatencyOrder {
            p50: sample.latency_p50_ms,
            p99: sample.latency_p99_ms,
        });
    }
    if sample.throughput_tps == 0 {
        return Err(ObservabilityError::ZeroThroughput);
    }
    Ok(())
}
