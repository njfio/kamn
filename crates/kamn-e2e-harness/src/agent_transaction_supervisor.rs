use std::path::Path;

use serde_json::Value;

use super::agent_transaction_evidence::AgentTransactionEvidencePaths;
use super::agent_transaction_finalize::finalize;
use super::agent_transaction_rpc::RpcGroup;
use super::agent_transaction_runtime::LocalRuntime;
use super::AgentTransactionDemoConfig;

const PROOF_ERROR: &str = "AGENT_TRANSACTION_PROOF_INVALID";

pub(super) fn run_supervised_registration(
    config: &AgentTransactionDemoConfig,
) -> Result<String, String> {
    let paths = match AgentTransactionEvidencePaths::prepare(config) {
        Ok(paths) => paths,
        Err(error) => return fail_no_go(config, error.as_str()),
    };
    execute_with_paths(config, &paths)
}

fn execute_with_paths(
    config: &AgentTransactionDemoConfig,
    paths: &AgentTransactionEvidencePaths,
) -> Result<String, String> {
    let mut runtime = match LocalRuntime::start(config) {
        Ok(runtime) => runtime,
        Err(error) => return fail_no_go(config, error.as_str()),
    };
    let mut group = match RpcGroup::spawn(config, paths) {
        Ok(group) => group,
        Err(error) => {
            runtime.cleanup();
            return fail_no_go(config, error.as_str());
        }
    };
    let result = run_phases(&mut group);
    group.cleanup();
    runtime.cleanup();
    match result {
        Ok(()) => match finalize(config, paths) {
            Ok(output) => Ok(output),
            Err(error) => fail_no_go(config, format!("{PROOF_ERROR}: {error}").as_str()),
        },
        Err(error) => fail_no_go(config, error.as_str()),
    }
}

fn run_phases(group: &mut RpcGroup) -> Result<(), String> {
    let registration = group.prompt(
        1,
        "Call kamn_live_agent_b_register exactly once, then stop.",
    )?;
    let provider_did = find_string(&registration, "did")
        .ok_or_else(|| "AGENT_TRANSACTION_CHILD_FAILED: Agent B omitted DID".to_owned())?;
    group.prompt(0, agent_a_create_prompt(provider_did).as_str())?;
    group.prompt(1, AGENT_B_ACCEPT_PROMPT)?;
    group.prompt(0, AGENT_A_FUND_PROMPT)?;
    group.prompt(1, AGENT_B_COMPLETE_PROMPT)?;
    group.prompt(0, AGENT_A_RELEASE_PROMPT)?;
    group.prompt(2, AGENT_C_VERIFY_PROMPT)?;
    Ok(())
}

fn agent_a_create_prompt(provider_did: &str) -> String {
    format!(
        "Call these tools exactly once in order: kamn_live_agent_a_register; \
kamn_live_agent_a_create_task with title 'KAMN evaluator transaction', description \
'Runtime-backed agent transaction', and provider_did '{provider_did}'; \
kamn_live_agent_a_publish_task_handoff. Then stop."
    )
}

const AGENT_B_ACCEPT_PROMPT: &str = "Call these tools exactly once in order: \
kamn_live_agent_b_receive_task_handoff; kamn_live_agent_b_accept_task; \
kamn_live_agent_b_write_task_receipt. Then stop.";

const AGENT_A_FUND_PROMPT: &str = "Call these tools exactly once in order: \
kamn_live_agent_a_wait_for_task_acceptance; kamn_live_agent_a_fund_escrow. Then stop.";

const AGENT_B_COMPLETE_PROMPT: &str = "Call these tools exactly once in order: \
kamn_live_agent_b_wait_for_escrow_funding; kamn_live_agent_b_complete_task; \
kamn_live_agent_b_query_participant_projection; \
kamn_live_agent_b_write_transaction_evidence. Then stop.";

const AGENT_A_RELEASE_PROMPT: &str = "Call these tools exactly once in order: \
kamn_live_agent_a_wait_for_task_completion; kamn_live_agent_a_release_escrow; \
kamn_live_agent_a_query_participant_projection; \
kamn_live_agent_a_write_transaction_evidence. Then stop.";

const AGENT_C_VERIFY_PROMPT: &str = "Call these tools exactly once in order: \
kamn_live_agent_c_register; kamn_live_agent_c_receive_task_handoff; \
kamn_live_agent_c_query_verifier_projection; \
kamn_live_agent_c_verify_restricted_task_observation; \
kamn_live_agent_c_write_transaction_evidence; \
kamn_live_verify_pi_transaction_actors. Then stop.";

fn find_string<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    match value {
        Value::Object(map) => map
            .get(field)
            .and_then(Value::as_str)
            .or_else(|| map.values().find_map(|item| find_string(item, field))),
        Value::Array(items) => items.iter().find_map(|item| find_string(item, field)),
        _ => None,
    }
}

fn fail_no_go(config: &AgentTransactionDemoConfig, error: &str) -> Result<String, String> {
    let latest = Path::new(config.output_root.as_str()).join("latest");
    std::fs::create_dir_all(&latest)
        .map_err(|_| "AGENT_TRANSACTION_PROOF_INVALID: NO-GO directory failed".to_owned())?;
    std::fs::write(
        latest.join("NO-GO.txt"),
        format!("decision=NO-GO\nreason={error}\n"),
    )
    .map_err(|_| "AGENT_TRANSACTION_PROOF_INVALID: NO-GO report failed".to_owned())?;
    Err(error.to_owned())
}
