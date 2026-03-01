//! Deterministic summary projections for live probe matrix reports.

use crate::{LiveProbeMatrixReport, LiveProbeMatrixStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Deterministic aggregate summary for one live probe matrix report.
pub struct LiveProbeMatrixSummary {
    /// Total number of matrix entries.
    pub total_entries: usize,
    /// Count of PASS entries.
    pub pass_entries: usize,
    /// Count of FAIL entries.
    pub fail_entries: usize,
    /// Count of SKIP entries.
    pub skip_entries: usize,
    /// Deterministic aggregate status across all rows.
    pub overall_status: Option<LiveProbeMatrixStatus>,
}

/// Projects one report into deterministic status counts and overall status.
pub fn project_live_probe_matrix_summary(report: &LiveProbeMatrixReport) -> LiveProbeMatrixSummary {
    let (pass_entries, fail_entries, skip_entries) = count_statuses(report);

    LiveProbeMatrixSummary {
        total_entries: report.entries().len(),
        pass_entries,
        fail_entries,
        skip_entries,
        overall_status: report.overall_status(),
    }
}

fn count_statuses(report: &LiveProbeMatrixReport) -> (usize, usize, usize) {
    let mut pass_entries = 0usize;
    let mut fail_entries = 0usize;
    let mut skip_entries = 0usize;

    for entry in report.entries() {
        match entry.status {
            LiveProbeMatrixStatus::Pass => pass_entries += 1,
            LiveProbeMatrixStatus::Fail => fail_entries += 1,
            LiveProbeMatrixStatus::Skip => skip_entries += 1,
        }
    }

    (pass_entries, fail_entries, skip_entries)
}
