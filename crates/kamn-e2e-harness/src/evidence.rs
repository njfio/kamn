use crate::ExecutionMode;

/// PRD evidence manifest schema marker.
pub const MANIFEST_SCHEMA_VERSION: &str = "kamn.e2e.evidence-manifest.v3";

/// Scenario result entry in an evidence manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioResult {
    /// Scenario ID marker.
    pub id: &'static str,
    /// Scenario status marker (`PASS`/`FAIL`/`SKIP`).
    pub status: &'static str,
}

/// Deterministic evidence manifest scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceManifest {
    /// Schema version marker.
    pub schema_version: &'static str,
    /// Selected execution mode.
    pub execution_mode: ExecutionMode,
    /// Scenario results.
    pub scenarios: Vec<ScenarioResult>,
}

impl EvidenceManifest {
    /// Builds a deterministic manifest with schema marker pinned.
    pub fn new(execution_mode: ExecutionMode, scenarios: Vec<ScenarioResult>) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            execution_mode,
            scenarios,
        }
    }
}
