use crate::evidence::MANIFEST_SCHEMA_VERSION;
use crate::verify::support::require_marker;

const MANIFEST_MARKERS: [(&str, &str); 17] = [
    (MANIFEST_SCHEMA_VERSION, "manifest schema version mismatch"),
    ("\"run_id\":", "manifest missing run_id"),
    ("\"started_at\":", "manifest missing started_at"),
    ("\"completed_at\":", "manifest missing completed_at"),
    ("\"duration_seconds\":", "manifest missing duration_seconds"),
    ("\"execution_mode\":", "manifest missing execution_mode"),
    ("\"infrastructure\":", "manifest missing infrastructure"),
    ("\"scenarios\":", "manifest missing scenarios"),
    ("\"summary\":", "manifest missing summary"),
    (
        "\"kolme_version\":",
        "manifest missing infrastructure.kolme_version",
    ),
    (
        "\"kamn_version\":",
        "manifest missing infrastructure.kamn_version",
    ),
    (
        "\"kamn_commit\":",
        "manifest missing infrastructure.kamn_commit",
    ),
    (
        "\"kamn_agent_lib_version\":",
        "manifest missing infrastructure.kamn_agent_lib_version",
    ),
    (
        "\"agent_runtime\":",
        "manifest missing infrastructure.agent_runtime",
    ),
    (
        "\"node_count\":",
        "manifest missing infrastructure.node_count",
    ),
    (
        "\"agent_count\":",
        "manifest missing infrastructure.agent_count",
    ),
    (
        "\"storage_backend\":",
        "manifest missing infrastructure.storage_backend",
    ),
];

const SUMMARY_MARKERS: [(&str, &str); 8] = [
    (
        "\"total_scenarios\":",
        "manifest missing summary.total_scenarios",
    ),
    ("\"passed\":", "manifest missing summary.passed"),
    ("\"failed\":", "manifest missing summary.failed"),
    ("\"skipped\":", "manifest missing summary.skipped"),
    (
        "\"kolme_blocks_produced\":",
        "manifest missing summary.kolme_blocks_produced",
    ),
    (
        "\"messages_exchanged\":",
        "manifest missing summary.messages_exchanged",
    ),
    (
        "\"proofs_anchored\":",
        "manifest missing summary.proofs_anchored",
    ),
    (
        "\"proofs_verified\":",
        "manifest missing summary.proofs_verified",
    ),
];

fn require_all(document: &str, markers: &[(&str, &str)]) -> Result<(), String> {
    for (marker, error) in markers {
        require_marker(document, marker, error)?;
    }
    Ok(())
}

/// Verifies a minimal JSON manifest payload using deterministic marker checks.
pub fn verify_manifest(manifest_json: &str) -> Result<(), String> {
    require_all(manifest_json, &MANIFEST_MARKERS)?;
    require_all(manifest_json, &SUMMARY_MARKERS)
}
