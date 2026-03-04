use super::{
    cli_value_parsers::{parse_proposal_candidate, parse_rejoin_attempt, parse_state_version_arg},
    ConfigError, ProposalCandidate, RejoinAttempt,
};

pub(super) struct PlanningRecoveryOptionState<'a> {
    pub(super) expected_state_version: &'a mut Option<u64>,
    pub(super) expected_state_hash: &'a mut Option<String>,
    pub(super) proposals: &'a mut Vec<ProposalCandidate>,
    pub(super) rejoin_attempts: &'a mut Vec<RejoinAttempt>,
}

pub(super) fn try_parse_planning_recovery_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut PlanningRecoveryOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--expected-state-version" => {
            set_expected_state_version(iter, state.expected_state_version).map(|_| true)
        }
        "--expected-state-hash" => {
            set_expected_state_hash(iter, state.expected_state_hash).map(|_| true)
        }
        "--proposal" => push_proposal(iter, state.proposals).map(|_| true),
        "--rejoin-attempt" => push_rejoin_attempt(iter, state.rejoin_attempts).map(|_| true),
        _ => Ok(false),
    }
}

fn set_expected_state_version(
    iter: &mut std::vec::IntoIter<String>,
    target: &mut Option<u64>,
) -> Result<(), ConfigError> {
    let value = read_required_value(iter, "--expected-state-version")?;
    *target = Some(parse_state_version_arg(&value)?);
    Ok(())
}

fn set_expected_state_hash(
    iter: &mut std::vec::IntoIter<String>,
    target: &mut Option<String>,
) -> Result<(), ConfigError> {
    *target = Some(read_required_value(iter, "--expected-state-hash")?);
    Ok(())
}

fn push_proposal(
    iter: &mut std::vec::IntoIter<String>,
    target: &mut Vec<ProposalCandidate>,
) -> Result<(), ConfigError> {
    let value = read_required_value(iter, "--proposal")?;
    target.push(parse_proposal_candidate(&value)?);
    Ok(())
}

fn push_rejoin_attempt(
    iter: &mut std::vec::IntoIter<String>,
    target: &mut Vec<RejoinAttempt>,
) -> Result<(), ConfigError> {
    let value = read_required_value(iter, "--rejoin-attempt")?;
    target.push(parse_rejoin_attempt(&value)?);
    Ok(())
}

fn read_required_value(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<String, ConfigError> {
    iter.next().ok_or(ConfigError::MissingArgumentValue(flag))
}
