use super::*;
use crate::zk_message_proofs::errors::ZkDesignError;

pub(super) fn validate_policy(policy: ZkEvaluationPolicy) -> Result<(), ZkDesignError> {
    if policy.max_verifier_latency_ms == 0 {
        return Err(ZkDesignError::InvalidPolicy(
            "max_verifier_latency_ms must be greater than zero".to_owned(),
        ));
    }
    if policy.max_proof_size_bytes == 0 {
        return Err(ZkDesignError::InvalidPolicy(
            "max_proof_size_bytes must be greater than zero".to_owned(),
        ));
    }
    if policy.max_engineering_weeks == 0 {
        return Err(ZkDesignError::InvalidPolicy(
            "max_engineering_weeks must be greater than zero".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_option(option: &ZkArchitectureOption) -> Result<(), ZkDesignError> {
    if option.name.trim().is_empty() {
        return invalid_option(&option.name, "name must not be empty");
    }
    if option.prover_latency_ms == 0 {
        return invalid_option(&option.name, "prover_latency_ms must be greater than zero");
    }
    if option.verifier_latency_ms == 0 {
        return invalid_option(
            &option.name,
            "verifier_latency_ms must be greater than zero",
        );
    }
    if option.proof_size_bytes == 0 {
        return invalid_option(&option.name, "proof_size_bytes must be greater than zero");
    }
    if option.estimated_engineering_weeks == 0 {
        return invalid_option(
            &option.name,
            "estimated_engineering_weeks must be greater than zero",
        );
    }
    Ok(())
}

fn invalid_option(option: &str, reason: &str) -> Result<(), ZkDesignError> {
    Err(ZkDesignError::InvalidOption {
        option: option.to_owned(),
        reason: reason.to_owned(),
    })
}
