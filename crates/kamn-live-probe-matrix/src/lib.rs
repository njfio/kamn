use std::collections::{BTreeMap, BTreeSet};
use tracing::debug;

/// Supported live probe execution modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveProbeMatrixMode {
    /// SDK-direct harness mode.
    SdkDirect,
    /// CLI-scripted harness mode.
    CliScripted,
    /// MCP-agent harness mode.
    McpTau,
}

impl LiveProbeMatrixMode {
    /// Returns canonical marker text for one mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SdkDirect => "sdk-direct",
            Self::CliScripted => "cli-scripted",
            Self::McpTau => "mcp-tau",
        }
    }
}

/// Scenario status marker used by live probe matrices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveProbeMatrixStatus {
    /// Scenario succeeded.
    Pass,
    /// Scenario failed.
    Fail,
    /// Scenario was intentionally skipped.
    Skip,
}

impl LiveProbeMatrixStatus {
    /// Returns canonical marker text for one status.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
        }
    }
}

/// One mode/scenario outcome row in the live probe matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveProbeMatrixEntry {
    /// Execution mode.
    pub mode: LiveProbeMatrixMode,
    /// Scenario identifier (`S-01`, `S-04`, ...).
    pub scenario_id: String,
    /// Outcome status for the mode/scenario pair.
    pub status: LiveProbeMatrixStatus,
}

impl LiveProbeMatrixEntry {
    /// Creates a validated matrix row.
    pub fn new(
        mode: LiveProbeMatrixMode,
        scenario_id: &str,
        status: LiveProbeMatrixStatus,
    ) -> Result<Self, LiveProbeMatrixError> {
        let normalized_scenario = scenario_id.trim();
        if normalized_scenario.is_empty() {
            debug!(
                mode = ?mode,
                status = ?status,
                "rejected live probe matrix row with empty scenario id"
            );
            return Err(LiveProbeMatrixError::EmptyScenarioId);
        }
        debug!(
            mode = ?mode,
            scenario_id = normalized_scenario,
            status = ?status,
            "validated live probe matrix row"
        );
        Ok(Self {
            mode,
            scenario_id: normalized_scenario.to_owned(),
            status,
        })
    }
}

/// Matrix construction and evaluation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveProbeMatrixError {
    /// Scenario id was empty after normalization.
    EmptyScenarioId,
    /// Duplicate mode/scenario row was provided.
    DuplicateModeScenario {
        /// Duplicated mode marker.
        mode: LiveProbeMatrixMode,
        /// Duplicated scenario id.
        scenario_id: String,
    },
}

impl std::fmt::Display for LiveProbeMatrixError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyScenarioId => write!(formatter, "scenario_id must not be empty"),
            Self::DuplicateModeScenario { mode, scenario_id } => write!(
                formatter,
                "duplicate live probe matrix row for mode={} scenario_id={}",
                mode.as_str(),
                scenario_id
            ),
        }
    }
}

impl std::error::Error for LiveProbeMatrixError {}

/// Validated live probe matrix with deterministic aggregate helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveProbeMatrixReport {
    entries: Vec<LiveProbeMatrixEntry>,
}

impl LiveProbeMatrixReport {
    /// Builds one validated matrix report.
    pub fn new(entries: Vec<LiveProbeMatrixEntry>) -> Result<Self, LiveProbeMatrixError> {
        let mut seen = BTreeSet::<(LiveProbeMatrixMode, String)>::new();
        for entry in &entries {
            let key = (entry.mode, entry.scenario_id.clone());
            if !seen.insert(key.clone()) {
                debug!(
                    mode = ?key.0,
                    scenario_id = key.1.as_str(),
                    "rejected duplicate live probe matrix row"
                );
                return Err(LiveProbeMatrixError::DuplicateModeScenario {
                    mode: key.0,
                    scenario_id: key.1,
                });
            }
        }
        debug!(
            entry_count = entries.len(),
            "validated live probe matrix report entries"
        );
        Ok(Self { entries })
    }

    /// Returns immutable entry slice.
    pub fn entries(&self) -> &[LiveProbeMatrixEntry] {
        self.entries.as_slice()
    }

    /// Returns status for one mode/scenario pair.
    pub fn status_for(
        &self,
        mode: LiveProbeMatrixMode,
        scenario_id: &str,
    ) -> Option<LiveProbeMatrixStatus> {
        let normalized_scenario = scenario_id.trim();
        if normalized_scenario.is_empty() {
            debug!(
                mode = ?mode,
                "status_for received empty scenario id and returned none"
            );
            return None;
        }
        let status = self
            .entries
            .iter()
            .find(|entry| entry.mode == mode && entry.scenario_id == normalized_scenario)
            .map(|entry| entry.status);
        debug!(
            mode = ?mode,
            scenario_id = normalized_scenario,
            status = ?status,
            "resolved live probe matrix status_for"
        );
        status
    }

    /// Returns deterministic aggregate status for one mode.
    pub fn mode_status(&self, mode: LiveProbeMatrixMode) -> Option<LiveProbeMatrixStatus> {
        let status = aggregate_status(
            self.entries
                .iter()
                .filter(|entry| entry.mode == mode)
                .map(|entry| entry.status),
        );
        debug!(
            mode = ?mode,
            status = ?status,
            "computed live probe matrix mode status"
        );
        status
    }

    /// Returns deterministic aggregate status across all rows.
    pub fn overall_status(&self) -> Option<LiveProbeMatrixStatus> {
        let status = aggregate_status(self.entries.iter().map(|entry| entry.status));
        debug!(
            status = ?status,
            entry_count = self.entries.len(),
            "computed live probe matrix overall status"
        );
        status
    }

    /// Returns per-mode deterministic aggregate status map.
    pub fn mode_status_map(&self) -> BTreeMap<LiveProbeMatrixMode, LiveProbeMatrixStatus> {
        let mut grouped = BTreeMap::<LiveProbeMatrixMode, Vec<LiveProbeMatrixStatus>>::new();
        for entry in &self.entries {
            grouped.entry(entry.mode).or_default().push(entry.status);
        }
        let mode_map = grouped
            .into_iter()
            .filter_map(|(mode, statuses)| aggregate_status(statuses).map(|status| (mode, status)))
            .collect::<BTreeMap<_, _>>();
        debug!(
            mode_count = mode_map.len(),
            "computed live probe matrix per-mode status map"
        );
        mode_map
    }
}

fn aggregate_status<I>(statuses: I) -> Option<LiveProbeMatrixStatus>
where
    I: IntoIterator<Item = LiveProbeMatrixStatus>,
{
    let statuses = statuses.into_iter().collect::<Vec<_>>();
    if statuses.is_empty() {
        return None;
    }
    if statuses.contains(&LiveProbeMatrixStatus::Fail) {
        return Some(LiveProbeMatrixStatus::Fail);
    }
    if statuses
        .iter()
        .all(|status| *status == LiveProbeMatrixStatus::Pass)
    {
        return Some(LiveProbeMatrixStatus::Pass);
    }
    if statuses
        .iter()
        .all(|status| *status == LiveProbeMatrixStatus::Skip)
    {
        return Some(LiveProbeMatrixStatus::Skip);
    }
    // Mixed PASS/SKIP is treated fail-closed because required coverage is incomplete.
    Some(LiveProbeMatrixStatus::Fail)
}

#[cfg(test)]
mod tests {
    use super::{
        LiveProbeMatrixEntry, LiveProbeMatrixMode, LiveProbeMatrixReport, LiveProbeMatrixStatus,
    };

    #[test]
    fn live_probe_matrix_entry_rejects_empty_scenario_id() {
        let error = LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "   ",
            LiveProbeMatrixStatus::Pass,
        )
        .expect_err("empty scenario id should fail closed");
        assert_eq!(error.to_string(), "scenario_id must not be empty");
    }

    #[test]
    fn live_probe_matrix_report_mode_and_overall_status_are_fail_closed_for_mixed_pass_skip() {
        let report = LiveProbeMatrixReport::new(vec![
            LiveProbeMatrixEntry::new(
                LiveProbeMatrixMode::SdkDirect,
                "S-01",
                LiveProbeMatrixStatus::Pass,
            )
            .expect("entry"),
            LiveProbeMatrixEntry::new(
                LiveProbeMatrixMode::SdkDirect,
                "S-04",
                LiveProbeMatrixStatus::Skip,
            )
            .expect("entry"),
        ])
        .expect("report");

        assert_eq!(
            report.mode_status(LiveProbeMatrixMode::SdkDirect),
            Some(LiveProbeMatrixStatus::Fail)
        );
        assert_eq!(report.overall_status(), Some(LiveProbeMatrixStatus::Fail));
    }

    #[test]
    fn live_probe_matrix_report_rejects_duplicate_mode_scenario_pair() {
        let duplicated = vec![
            LiveProbeMatrixEntry::new(
                LiveProbeMatrixMode::McpTau,
                "S-06",
                LiveProbeMatrixStatus::Pass,
            )
            .expect("entry"),
            LiveProbeMatrixEntry::new(
                LiveProbeMatrixMode::McpTau,
                "S-06",
                LiveProbeMatrixStatus::Fail,
            )
            .expect("entry"),
        ];
        let error =
            LiveProbeMatrixReport::new(duplicated).expect_err("duplicate row should fail closed");
        assert!(error
            .to_string()
            .contains("duplicate live probe matrix row"));
    }
}
