use crate::ExecutionMode;

/// PRD evidence manifest schema marker.
pub const MANIFEST_SCHEMA_VERSION: &str = "kamn.e2e.evidence-manifest.v3";

/// Infrastructure markers in an evidence manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceInfrastructure {
    /// Kolme runtime version marker.
    pub kolme_version: String,
    /// KAMN version marker.
    pub kamn_version: String,
    /// KAMN commit marker.
    pub kamn_commit: String,
    /// Shared agent library version marker.
    pub kamn_agent_lib_version: String,
    /// Agent runtime marker.
    pub agent_runtime: String,
    /// Number of KAMN nodes in the run.
    pub node_count: u64,
    /// Number of test agents in the run.
    pub agent_count: u64,
    /// Storage backend marker.
    pub storage_backend: String,
}

/// Scenario result entry in an evidence manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioResult {
    /// Scenario ID marker.
    pub id: String,
    /// Human-readable scenario name.
    pub name: String,
    /// Scenario status marker (`PASS`/`FAIL`/`SKIP`).
    pub status: String,
    /// Scenario duration marker.
    pub duration_seconds: u64,
    /// Evidence file globs linked to this scenario.
    pub evidence_files: Vec<String>,
    /// Number of verifiable outputs in this scenario.
    pub verifiable_outputs: u64,
}

/// Top-level summary counters in an evidence manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceSummary {
    /// Total scenarios planned for run.
    pub total_scenarios: u64,
    /// Passing scenario count.
    pub passed: u64,
    /// Failing scenario count.
    pub failed: u64,
    /// Skipped scenario count.
    pub skipped: u64,
    /// Produced Kolme block count.
    pub kolme_blocks_produced: u64,
    /// Message exchange count.
    pub messages_exchanged: u64,
    /// Anchored proof count.
    pub proofs_anchored: u64,
    /// Verified proof count.
    pub proofs_verified: u64,
}

/// Deterministic evidence manifest scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceManifest {
    /// Schema version marker.
    pub schema_version: &'static str,
    /// Run identifier.
    pub run_id: String,
    /// Run start timestamp.
    pub started_at: String,
    /// Run completion timestamp.
    pub completed_at: String,
    /// Total run duration.
    pub duration_seconds: u64,
    /// Selected execution mode.
    pub execution_mode: ExecutionMode,
    /// Infrastructure metadata.
    pub infrastructure: EvidenceInfrastructure,
    /// Scenario results.
    pub scenarios: Vec<ScenarioResult>,
    /// Summary counters.
    pub summary: EvidenceSummary,
}

impl EvidenceManifest {
    /// Builds a PRD section-8.2 contract manifest.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: String,
        started_at: String,
        completed_at: String,
        duration_seconds: u64,
        execution_mode: ExecutionMode,
        infrastructure: EvidenceInfrastructure,
        scenarios: Vec<ScenarioResult>,
        summary: EvidenceSummary,
    ) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            run_id,
            started_at,
            completed_at,
            duration_seconds,
            execution_mode,
            infrastructure,
            scenarios,
            summary,
        }
    }
}
