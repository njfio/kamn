use std::path::Path;

use super::artifact_digest::ThreeAgentArtifactDigests;
use super::command_config::MvpDemoCommandConfig;
use super::devnet_settlement::DevnetSettlementAttempt;
use super::live_task_binding::LiveTaskBinding;
use super::report::DemoReportInput;

pub(super) fn build_report_input<'a>(
    config: &'a MvpDemoCommandConfig,
    output_root: &'a Path,
    run_id: &'a str,
    settlement: &'a DevnetSettlementAttempt,
    binding: Option<&'a LiveTaskBinding>,
    artifact_digests: Option<&'a ThreeAgentArtifactDigests>,
) -> DemoReportInput<'a> {
    DemoReportInput {
        run_id,
        devnet_mode: config.devnet_mode.as_str(),
        solana_rpc_url: config.solana_rpc_url.as_deref(),
        output_root,
        devnet_settlement: settlement.evidence.as_ref(),
        live_task_binding: binding,
        devnet_no_go_reason: settlement.no_go_reason.as_deref(),
        agent_harness_evidence_path: config.agent_harness_evidence_path.as_deref(),
        three_agent_artifact_digests: artifact_digests,
    }
}
