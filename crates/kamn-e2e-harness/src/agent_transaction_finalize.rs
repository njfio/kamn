use super::agent_transaction_devnet_evidence::collect_actor_settlement_evidence;
use super::agent_transaction_evidence::AgentTransactionEvidencePaths;
use super::{
    build_runtime_receipt_chain_from_actor_paths, execute_mvp_demo_contract,
    execute_verify_mvp_demo_contract, AgentTransactionDemoConfig, LiveTaskEvidencePaths,
    MvpDemoCommandConfig, VerifyMvpDemoCommandConfig,
};

pub(super) fn finalize(
    config: &AgentTransactionDemoConfig,
    paths: &AgentTransactionEvidencePaths,
) -> Result<String, String> {
    build_runtime_receipt_chain_from_actor_paths(&paths.actors)?;
    let mut demo = demo_config(config, paths);
    if demo.devnet_settlement_command.is_none() {
        let evidence = collect_actor_settlement_evidence(config, paths)?;
        demo.devnet_settlement_command =
            Some(vec!["cat".to_owned(), evidence.display().to_string()]);
    }
    let report = execute_mvp_demo_contract(&demo)?;
    execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
        report: format!("{}/latest/proof/report.json", config.output_root),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: Some(paths.actors.clone()),
    })?;
    Ok(format!(
        "{{\"decision\":\"GO\",\"proof_schema\":\"kamn.mvp.runtime-receipt-chain.v1\",\"report\":{report}}}"
    ))
}

fn demo_config(
    config: &AgentTransactionDemoConfig,
    paths: &AgentTransactionEvidencePaths,
) -> MvpDemoCommandConfig {
    MvpDemoCommandConfig {
        output_root: config.output_root.clone(),
        devnet_mode: config.devnet_mode.clone(),
        solana_rpc_url: Some(config.solana_rpc_url.clone()),
        devnet_settlement_command: config.devnet_settlement_command.clone(),
        localhost_signed_demo_command: config.localhost_signed_demo_command.clone(),
        service_api_vertical_slice_command: config.service_api_vertical_slice_command.clone(),
        service_api_websocket_command: config.service_api_websocket_command.clone(),
        agent_harness_evidence_path: None,
        live_task_evidence: Some(LiveTaskEvidencePaths {
            handoff: paths.handoff.clone(),
            agent_a_receipt: paths.agent_a_receipt.clone(),
            agent_b_receipt: paths.agent_b_receipt.clone(),
            agent_c_observation: paths.agent_c_observation.clone(),
        }),
        pi_transaction_actor_paths: Some(paths.actors.clone()),
    }
}
