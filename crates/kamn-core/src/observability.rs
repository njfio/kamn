use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilityMetric {
    LatencyP50,
    LatencyP99,
    Throughput,
    ErrorRate,
    Availability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilitySeverity {
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilityHealth {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilitySample {
    pub latency_p50_ms: u64,
    pub latency_p99_ms: u64,
    pub throughput_tps: u64,
    pub error_rate_pct: f64,
    pub availability_pct: f64,
    pub timestamp_epoch_s: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilitySloProfile {
    pub max_latency_p50_ms: u64,
    pub max_latency_p99_ms: u64,
    pub min_throughput_tps: u64,
    pub max_error_rate_pct: f64,
    pub min_availability_pct: f64,
}

impl ObservabilitySloProfile {
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
pub struct ObservabilityAlert {
    pub metric: ObservabilityMetric,
    pub severity: ObservabilitySeverity,
    pub observed: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityReport {
    pub sample: ObservabilitySample,
    pub overall_health: ObservabilityHealth,
    pub alerts: Vec<ObservabilityAlert>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilitySnapshot {
    pub total_samples: usize,
    pub healthy_samples: usize,
    pub degraded_samples: usize,
    pub critical_samples: usize,
    pub latest_health: ObservabilityHealth,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityMonitor {
    profile: ObservabilitySloProfile,
    history: Vec<ObservabilityReport>,
}

impl ObservabilityMonitor {
    pub fn new(profile: ObservabilitySloProfile) -> Self {
        Self {
            profile,
            history: Vec::new(),
        }
    }

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
pub enum ObservabilityError {
    InvalidPercentage { field: &'static str, value: f64 },
    InvalidLatencyOrder { p50: u64, p99: u64 },
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
