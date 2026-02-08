use crate::observability::ObservabilitySample;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceMetric {
    LatencyP50,
    LatencyP99,
    Throughput,
    Availability,
}

impl PerformanceMetric {
    fn remediation_hint(self) -> &'static str {
        match self {
            Self::LatencyP50 => {
                "Reduce average queue depth by tuning listener batching and message routing."
            }
            Self::LatencyP99 => {
                "Investigate processor backlog and queue starvation before approval fan-out."
            }
            Self::Throughput => {
                "Increase listener and approver parallelism and reduce serialization overhead."
            }
            Self::Availability => {
                "Harden failover probes and standby promotion thresholds to prevent downtime."
            }
        }
    }

    fn bottleneck_priority(self) -> u8 {
        match self {
            Self::Throughput => 0,
            Self::LatencyP99 => 1,
            Self::Availability => 2,
            Self::LatencyP50 => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrdPerformanceTargets {
    pub max_latency_p50_ms: u64,
    pub max_latency_p99_ms: u64,
    pub min_throughput_tps: u64,
    pub min_availability_pct: f64,
}

impl PrdPerformanceTargets {
    pub fn v13_2() -> Self {
        Self {
            max_latency_p50_ms: 100,
            max_latency_p99_ms: 500,
            min_throughput_tps: 10_000,
            min_availability_pct: 99.9,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceSample {
    pub latency_p50_ms: u64,
    pub latency_p99_ms: u64,
    pub throughput_tps: u64,
    pub availability_pct: f64,
    pub timestamp_epoch_s: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceAggregate {
    pub latency_p50_ms: u64,
    pub latency_p99_ms: u64,
    pub throughput_tps: u64,
    pub availability_pct: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceMetricResult {
    pub metric: PerformanceMetric,
    pub observed: f64,
    pub threshold: f64,
    pub met: bool,
    pub deviation_pct: f64,
    pub remediation: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceRunReport {
    pub run_id: String,
    pub sample_count: usize,
    pub aggregate: PerformanceAggregate,
    pub results: Vec<PerformanceMetricResult>,
    pub meets_targets: bool,
}

impl PerformanceRunReport {
    pub fn metric_result(&self, metric: PerformanceMetric) -> Option<&PerformanceMetricResult> {
        self.results.iter().find(|result| result.metric == metric)
    }

    pub fn bottlenecks(&self) -> Vec<PerformanceMetric> {
        let mut failed: Vec<&PerformanceMetricResult> =
            self.results.iter().filter(|result| !result.met).collect();
        failed.sort_by(|left, right| {
            left.metric
                .bottleneck_priority()
                .cmp(&right.metric.bottleneck_priority())
                .then_with(|| right.deviation_pct.total_cmp(&left.deviation_pct))
        });

        failed.into_iter().map(|result| result.metric).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceRunError {
    EmptySamples,
    InvalidAvailability { value: f64 },
    InvalidLatencyOrder { p50: u64, p99: u64 },
    ZeroThroughput,
}

impl fmt::Display for PerformanceRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySamples => write!(f, "at least one benchmark sample is required"),
            Self::InvalidAvailability { value } => {
                write!(
                    f,
                    "availability_pct must be between 0 and 100, found {value}"
                )
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

impl std::error::Error for PerformanceRunError {}

pub fn evaluate_performance_run(
    run_id: impl Into<String>,
    samples: &[PerformanceSample],
    targets: &PrdPerformanceTargets,
) -> Result<PerformanceRunReport, PerformanceRunError> {
    if samples.is_empty() {
        return Err(PerformanceRunError::EmptySamples);
    }

    for sample in samples {
        validate_sample(sample)?;
    }

    let aggregate = aggregate_samples(samples);
    let results = vec![
        evaluate_upper_exclusive(
            PerformanceMetric::LatencyP50,
            aggregate.latency_p50_ms as f64,
            targets.max_latency_p50_ms as f64,
        ),
        evaluate_upper_exclusive(
            PerformanceMetric::LatencyP99,
            aggregate.latency_p99_ms as f64,
            targets.max_latency_p99_ms as f64,
        ),
        evaluate_lower_inclusive(
            PerformanceMetric::Throughput,
            aggregate.throughput_tps as f64,
            targets.min_throughput_tps as f64,
        ),
        evaluate_lower_inclusive(
            PerformanceMetric::Availability,
            aggregate.availability_pct,
            targets.min_availability_pct,
        ),
    ];

    let meets_targets = results.iter().all(|result| result.met);

    Ok(PerformanceRunReport {
        run_id: run_id.into(),
        sample_count: samples.len(),
        aggregate,
        results,
        meets_targets,
    })
}

pub fn evaluate_performance_from_observability(
    run_id: impl Into<String>,
    samples: &[ObservabilitySample],
    targets: &PrdPerformanceTargets,
) -> Result<PerformanceRunReport, PerformanceRunError> {
    let converted: Vec<PerformanceSample> = samples
        .iter()
        .map(|sample| PerformanceSample {
            latency_p50_ms: sample.latency_p50_ms,
            latency_p99_ms: sample.latency_p99_ms,
            throughput_tps: sample.throughput_tps,
            availability_pct: sample.availability_pct,
            timestamp_epoch_s: sample.timestamp_epoch_s,
        })
        .collect();

    evaluate_performance_run(run_id, &converted, targets)
}

fn evaluate_upper_exclusive(
    metric: PerformanceMetric,
    observed: f64,
    threshold: f64,
) -> PerformanceMetricResult {
    let met = observed < threshold;
    let deviation_pct = if met {
        0.0
    } else {
        ((observed - threshold) / threshold) * 100.0
    };

    PerformanceMetricResult {
        metric,
        observed,
        threshold,
        met,
        deviation_pct,
        remediation: metric.remediation_hint(),
    }
}

fn evaluate_lower_inclusive(
    metric: PerformanceMetric,
    observed: f64,
    threshold: f64,
) -> PerformanceMetricResult {
    let met = observed >= threshold;
    let deviation_pct = if met {
        0.0
    } else {
        ((threshold - observed) / threshold) * 100.0
    };

    PerformanceMetricResult {
        metric,
        observed,
        threshold,
        met,
        deviation_pct,
        remediation: metric.remediation_hint(),
    }
}

fn aggregate_samples(samples: &[PerformanceSample]) -> PerformanceAggregate {
    let mut p50_values: Vec<u64> = samples.iter().map(|sample| sample.latency_p50_ms).collect();
    p50_values.sort_unstable();

    PerformanceAggregate {
        latency_p50_ms: median_u64(&p50_values),
        latency_p99_ms: samples
            .iter()
            .map(|sample| sample.latency_p99_ms)
            .max()
            .unwrap_or(0),
        throughput_tps: samples
            .iter()
            .map(|sample| sample.throughput_tps)
            .min()
            .unwrap_or(0),
        availability_pct: samples
            .iter()
            .map(|sample| sample.availability_pct)
            .fold(100.0, f64::min),
    }
}

fn median_u64(sorted_values: &[u64]) -> u64 {
    let mid = sorted_values.len() / 2;
    if sorted_values.len() % 2 == 1 {
        sorted_values[mid]
    } else {
        ((sorted_values[mid - 1] as u128 + sorted_values[mid] as u128) / 2) as u64
    }
}

fn validate_sample(sample: &PerformanceSample) -> Result<(), PerformanceRunError> {
    if !(0.0..=100.0).contains(&sample.availability_pct) {
        return Err(PerformanceRunError::InvalidAvailability {
            value: sample.availability_pct,
        });
    }

    if sample.latency_p99_ms < sample.latency_p50_ms {
        return Err(PerformanceRunError::InvalidLatencyOrder {
            p50: sample.latency_p50_ms,
            p99: sample.latency_p99_ms,
        });
    }

    if sample.throughput_tps == 0 {
        return Err(PerformanceRunError::ZeroThroughput);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_even_size_rounds_down_integer_average() {
        assert_eq!(median_u64(&[20, 40, 60, 100]), 50);
    }

    #[test]
    fn latency_threshold_is_strictly_less_than_target() {
        let result = evaluate_upper_exclusive(PerformanceMetric::LatencyP50, 100.0, 100.0);
        assert!(!result.met);
        assert_eq!(result.deviation_pct, 0.0);
    }

    #[test]
    fn throughput_threshold_is_inclusive() {
        let result = evaluate_lower_inclusive(PerformanceMetric::Throughput, 10_000.0, 10_000.0);
        assert!(result.met);
        assert_eq!(result.deviation_pct, 0.0);
    }

    #[test]
    fn validate_rejects_bad_availability() {
        let sample = PerformanceSample {
            latency_p50_ms: 10,
            latency_p99_ms: 20,
            throughput_tps: 10,
            availability_pct: 101.0,
            timestamp_epoch_s: 1,
        };

        assert_eq!(
            validate_sample(&sample),
            Err(PerformanceRunError::InvalidAvailability { value: 101.0 })
        );
    }
}
