use super::AgentTransactionDemoConfig;

/// Role assigned to one independent Pi transaction process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTransactionRole {
    /// Task creator, escrow funder, and release authorizer.
    AgentA,
    /// Task provider and completion author.
    AgentB,
    /// Restricted independent verifier.
    AgentC,
}

/// Builds one persistent, role-bounded Pi RPC command.
pub fn build_pi_actor_command(
    config: &AgentTransactionDemoConfig,
    role: AgentTransactionRole,
) -> Vec<String> {
    let mut command = base_command(config);
    command.extend([
        "--tools".to_owned(),
        role_tools(role).join(","),
        "--name".to_owned(),
        role_name(role).to_owned(),
    ]);
    command
}

fn base_command(config: &AgentTransactionDemoConfig) -> Vec<String> {
    [
        config.pi_binary.as_str(),
        "--mode",
        "rpc",
        "--provider",
        config.pi_provider.as_str(),
        "--model",
        config.pi_model.as_str(),
        "--thinking",
        "high",
        "--no-session",
        "--approve",
        "--no-extensions",
        "--extension",
        config.pi_extension.as_str(),
        "--no-skills",
        "--no-prompt-templates",
        "--no-context-files",
        "--no-builtin-tools",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn role_tools(role: AgentTransactionRole) -> &'static [&'static str] {
    match role {
        AgentTransactionRole::AgentA => AGENT_A_TOOLS,
        AgentTransactionRole::AgentB => AGENT_B_TOOLS,
        AgentTransactionRole::AgentC => AGENT_C_TOOLS,
    }
}

const AGENT_A_TOOLS: &[&str] = &[
    "kamn_live_agent_a_register",
    "kamn_live_agent_a_create_task",
    "kamn_live_agent_a_publish_task_handoff",
    "kamn_live_agent_a_wait_for_task_acceptance",
    "kamn_live_agent_a_fund_escrow",
    "kamn_live_agent_a_wait_for_task_completion",
    "kamn_live_agent_a_release_escrow",
    "kamn_live_agent_a_query_participant_projection",
    "kamn_live_agent_a_write_transaction_evidence",
];

const AGENT_B_TOOLS: &[&str] = &[
    "kamn_live_agent_b_register",
    "kamn_live_agent_b_receive_task_handoff",
    "kamn_live_agent_b_accept_task",
    "kamn_live_agent_b_write_task_receipt",
    "kamn_live_agent_b_wait_for_escrow_funding",
    "kamn_live_agent_b_complete_task",
    "kamn_live_agent_b_query_participant_projection",
    "kamn_live_agent_b_write_transaction_evidence",
];

const AGENT_C_TOOLS: &[&str] = &[
    "kamn_live_agent_c_register",
    "kamn_live_agent_c_receive_task_handoff",
    "kamn_live_agent_c_query_verifier_projection",
    "kamn_live_agent_c_write_transaction_evidence",
    "kamn_live_verify_pi_transaction_actors",
];

fn role_name(role: AgentTransactionRole) -> &'static str {
    match role {
        AgentTransactionRole::AgentA => "kamn-mvp-agent-a",
        AgentTransactionRole::AgentB => "kamn-mvp-agent-b",
        AgentTransactionRole::AgentC => "kamn-mvp-agent-c",
    }
}
