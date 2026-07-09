use std::path::Path;

use super::{actor_receipts, canonical_receipts, three_agent};

pub(crate) fn agent_artifact(root: &Path, private_visible: bool, settlement_label: &str) -> String {
    agent_artifact_with_surface(root, private_visible, settlement_label, "mcp-tools")
}

pub(crate) fn agent_artifact_without_three_agent_boundary(root: &Path) -> String {
    agent_artifact_for_report_with_surface(
        root.join("proof/report.json")
            .display()
            .to_string()
            .as_str(),
        false,
        "devnet-backed",
        "pi-extension-tools",
        three_agent::NO_THREE_AGENT_BOUNDARY,
    )
}

pub(crate) fn agent_artifact_with_surface(
    root: &Path,
    private_visible: bool,
    settlement_label: &str,
    execution_surface: &str,
) -> String {
    agent_artifact_for_report_with_surface(
        root.join("proof/report.json")
            .display()
            .to_string()
            .as_str(),
        private_visible,
        settlement_label,
        execution_surface,
        three_agent::absent_boundary(),
    )
}

pub(crate) fn agent_latest_artifact(root: &Path) -> String {
    agent_latest_artifact_with_surface(root, "mcp-tools")
}

pub(crate) fn agent_latest_artifact_with_surface(root: &Path, execution_surface: &str) -> String {
    agent_artifact_for_report_with_surface(
        root.join("latest/proof/report.json")
            .display()
            .to_string()
            .as_str(),
        false,
        "devnet-backed",
        execution_surface,
        three_agent::absent_boundary(),
    )
}

pub(crate) fn agent_artifact_with_three_agent_boundary(root: &Path) -> String {
    agent_artifact_for_report_with_surface(
        root.join("proof/report.json")
            .display()
            .to_string()
            .as_str(),
        false,
        "devnet-backed",
        "pi-extension-tools",
        three_agent::valid_boundary(),
    )
}

pub(crate) fn agent_artifact_with_three_agent_actor_rehearsal(root: &Path) -> String {
    append_object_field(
        agent_artifact_with_three_agent_boundary(root),
        three_agent::valid_actor_rehearsal(root),
    )
}

pub(crate) fn agent_artifact_with_three_agent_actor_receipts(root: &Path) -> String {
    append_object_field(
        agent_artifact_with_three_agent_actor_rehearsal(root),
        actor_receipts::valid_actor_receipts(root),
    )
}

pub(crate) fn agent_artifact_with_canonical_observation_receipts(root: &Path) -> String {
    append_object_field(
        agent_artifact_with_three_agent_actor_receipts(root),
        canonical_receipts::observation_receipts(root),
    )
}

fn append_object_field(mut json: String, field: String) -> String {
    json.pop();
    json.push_str(field.as_str());
    json.push('}');
    json
}

fn agent_artifact_for_report_with_surface(
    report_path: &str,
    private_visible: bool,
    settlement_label: &str,
    execution_surface: &str,
    three_agent_boundary: &str,
) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.agent-harness-evidence.v1","harness":"mcp-agent","execution_surface":"{}","report_path":"{}","verifier_status":"PASS","participant_agents":["agent_a","agent_b","agent_c_verifier"],"tool_markers":["register","create_task","fund_escrow","release_escrow","verify_proof"],"claim_boundaries":{{"settlement_claim_label":"{}","dry_run_counted_as_success":false,"placeholder_counted_as_success":false,"verifier_private_view_visible":{}}}{}}}"#,
        execution_surface, report_path, settlement_label, private_visible, three_agent_boundary
    )
}
