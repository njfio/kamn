use std::path::Path;

use super::artifact_digest::ThreeAgentArtifactDigests;
use super::command_config::MvpDemoCommandConfig;
use super::devnet_settlement::{
    collect_devnet_settlement_evidence, DevnetSettlementAttempt, DevnetSettlementInput,
};
use super::live_task_binding::{create_live_task_binding, LiveTaskBinding};
use super::three_agent_receipts::write_three_agent_receipts;
use super::three_agent_transcript::write_three_agent_transcript;
use super::three_agent_views::write_three_agent_views;

pub(super) struct BoundSettlement {
    pub(super) binding: Option<LiveTaskBinding>,
    pub(super) settlement: DevnetSettlementAttempt,
    pub(super) artifact_digests: Option<ThreeAgentArtifactDigests>,
}

pub(super) fn create_bound_settlement(
    config: &MvpDemoCommandConfig,
    run_id: &str,
    run_dir: &Path,
) -> Result<BoundSettlement, String> {
    let binding = create_binding(config, run_dir)?;
    let settlement = create_settlement(config, run_dir, binding.as_ref())?;
    let artifact_digests =
        create_three_agent_artifacts(run_id, &settlement, binding.as_ref(), run_dir)?;
    Ok(BoundSettlement {
        binding,
        settlement,
        artifact_digests,
    })
}

fn create_binding(
    config: &MvpDemoCommandConfig,
    run_dir: &Path,
) -> Result<Option<LiveTaskBinding>, String> {
    config
        .live_task_evidence
        .as_ref()
        .map(|paths| create_live_task_binding(paths, run_dir))
        .transpose()
}

fn create_settlement(
    config: &MvpDemoCommandConfig,
    run_dir: &Path,
    binding: Option<&LiveTaskBinding>,
) -> Result<DevnetSettlementAttempt, String> {
    if config.devnet_mode != "required" {
        write_skipped_log(run_dir, "devnet_mode_optional")?;
        return Ok(DevnetSettlementAttempt::default());
    }
    collect_devnet_settlement_evidence(&DevnetSettlementInput {
        command: config.devnet_settlement_command.as_deref(),
        solana_rpc_url: config.solana_rpc_url.as_deref(),
        run_dir,
        live_task_binding: binding,
    })
}

fn create_three_agent_artifacts(
    run_id: &str,
    settlement: &DevnetSettlementAttempt,
    binding: Option<&LiveTaskBinding>,
    run_dir: &Path,
) -> Result<Option<ThreeAgentArtifactDigests>, String> {
    let (Some(binding), Some(evidence)) = (binding, settlement.evidence.as_ref()) else {
        return Ok(None);
    };
    let views = write_three_agent_views(run_id, evidence, binding, run_dir)?;
    let receipts = write_three_agent_receipts(run_id, evidence, binding, run_dir, &views)?;
    let transcript = write_three_agent_transcript(run_id, evidence, binding, run_dir, &views)?;
    Ok(Some(ThreeAgentArtifactDigests {
        transcript,
        views,
        receipts,
    }))
}

fn write_skipped_log(run_dir: &Path, reason: &str) -> Result<(), String> {
    let path = run_dir.join("proof/devnet-settlement-output.txt");
    std::fs::write(
        &path,
        format!("devnet_settlement_status=SKIP reason={reason}\n"),
    )
    .map_err(|error| {
        format!(
            "failed to write devnet settlement skip log {}: {error}",
            path.display()
        )
    })
}
