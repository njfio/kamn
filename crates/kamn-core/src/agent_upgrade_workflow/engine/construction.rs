use std::collections::BTreeSet;

use crate::{GovernanceWorkflow, VersionUpgradeOrchestrator};

use crate::agent_upgrade_workflow::{
    support::{
        validate_did, AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_PROPOSER_DID_REASON_CODE,
        AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_VALIDATOR_DID_REASON_CODE,
    },
    AgentDrivenUpgradeWorkflow, AgentUpgradeWorkflowConfig, AgentUpgradeWorkflowError,
};

impl AgentDrivenUpgradeWorkflow {
    /// Construct a workflow instance after validating config invariants and DID allowlists.
    pub fn new(config: AgentUpgradeWorkflowConfig) -> Result<Self, AgentUpgradeWorkflowError> {
        validate_config_invariants(&config)?;
        let allowlists = collect_allowlists(&config)?;
        build_workflow(config, allowlists)
    }
}

fn collect_allowlists(
    config: &AgentUpgradeWorkflowConfig,
) -> Result<(BTreeSet<String>, BTreeSet<String>), AgentUpgradeWorkflowError> {
    let proposers = collect_allowed_dids(
        config.allowed_agent_proposers.clone(),
        "config.allowed_agent_proposers[]",
        AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_PROPOSER_DID_REASON_CODE,
    )?;
    let validators = collect_allowed_dids(
        config.allowed_validator_voters.clone(),
        "config.allowed_validator_voters[]",
        AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_VALIDATOR_DID_REASON_CODE,
    )?;
    Ok((proposers, validators))
}

fn build_workflow(
    config: AgentUpgradeWorkflowConfig,
    (allowed_agent_proposers, allowed_validator_voters): (BTreeSet<String>, BTreeSet<String>),
) -> Result<AgentDrivenUpgradeWorkflow, AgentUpgradeWorkflowError> {
    let orchestrator = VersionUpgradeOrchestrator::new(&config.current_version)
        .map_err(AgentUpgradeWorkflowError::UpgradeOrchestration)?;
    Ok(AgentDrivenUpgradeWorkflow {
        allowed_agent_proposers,
        allowed_validator_voters,
        required_human_reviews: config.required_human_reviews,
        required_validator_quorum: config.required_validator_quorum,
        min_activation_delay_secs: config.min_activation_delay_secs,
        governance: GovernanceWorkflow::new(),
        orchestrator,
        proposals: Default::default(),
        events: Default::default(),
    })
}

fn validate_config_invariants(
    config: &AgentUpgradeWorkflowConfig,
) -> Result<(), AgentUpgradeWorkflowError> {
    if config.required_human_reviews == 0 {
        return Err(AgentUpgradeWorkflowError::InvalidRequiredHumanReviews(0));
    }
    if config.required_validator_quorum == 0 {
        return Err(AgentUpgradeWorkflowError::InvalidRequiredValidatorQuorum(0));
    }
    if config.min_activation_delay_secs == 0 {
        return Err(AgentUpgradeWorkflowError::InvalidMinActivationDelaySecs(0));
    }
    if config.allowed_agent_proposers.is_empty() {
        return Err(AgentUpgradeWorkflowError::MissingAllowedAgentProposers);
    }
    if config.allowed_validator_voters.is_empty() {
        return Err(AgentUpgradeWorkflowError::MissingAllowedValidatorVoters);
    }
    Ok(())
}

fn collect_allowed_dids(
    values: Vec<String>,
    field: &'static str,
    reason_code: &'static str,
) -> Result<BTreeSet<String>, AgentUpgradeWorkflowError> {
    let mut collected = BTreeSet::new();
    for value in values {
        validate_did(&value, field, reason_code)?;
        collected.insert(value);
    }
    Ok(collected)
}
