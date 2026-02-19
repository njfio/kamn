use std::collections::BTreeSet;
use std::env;

use kamn_core::{ConfigError, SecureSignerProvider, SignerKeyRole};

use super::{
    KolmeLiveSignerPreflightReadiness, KolmeLiveSignerSelection, KOLME_LIVE_SIGNER_KEY_REF_ENV,
    KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV, KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL,
    KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL, KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV, KOLME_LIVE_SIGNER_PROFILE_ENV,
    KOLME_LIVE_SIGNER_PROFILE_PRIMARY, KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
};

const KOLME_LIVE_SIGNER_PREVIOUS_PROFILE_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE";
const KOLME_LIVE_SIGNER_ROTATION_EPOCH_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH";
const KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH";
const KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS";
const KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS";
const KOLME_LIVE_SIGNER_QUORUM_LINKAGE_CONTRACT_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignerRotationFreshnessOutcome {
    Fresh,
    FailoverEpochStale {
        rotation_epoch: u64,
        previous_rotation_epoch: u64,
    },
    NonFailoverEpochRegressed {
        rotation_epoch: u64,
        previous_rotation_epoch: u64,
    },
}

pub(crate) fn evaluate_signer_rotation_freshness(
    failover_active: bool,
    rotation_epoch: u64,
    previous_rotation_epoch: u64,
) -> SignerRotationFreshnessOutcome {
    if failover_active && rotation_epoch <= previous_rotation_epoch {
        return SignerRotationFreshnessOutcome::FailoverEpochStale {
            rotation_epoch,
            previous_rotation_epoch,
        };
    }
    if !failover_active && rotation_epoch < previous_rotation_epoch {
        return SignerRotationFreshnessOutcome::NonFailoverEpochRegressed {
            rotation_epoch,
            previous_rotation_epoch,
        };
    }
    SignerRotationFreshnessOutcome::Fresh
}

pub(crate) fn normalize_kolme_live_signer_profile_selector(
    value: &str,
) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-profile must not be empty".to_owned(),
        ));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(KOLME_LIVE_SIGNER_PROFILE_PRIMARY),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(KOLME_LIVE_SIGNER_PROFILE_SECONDARY),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-profile must be one of {KOLME_LIVE_SIGNER_PROFILE_PRIMARY}, {KOLME_LIVE_SIGNER_PROFILE_SECONDARY}; found {trimmed}"
        ))),
    }
}

pub(crate) fn normalize_kolme_live_signer_key_source(
    value: &str,
) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-key-source must not be empty".to_owned(),
        ));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL => Ok(KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL),
        KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL => {
            Ok(KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL)
        }
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-key-source must be one of {KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL}, {KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL}; found {trimmed}"
        ))),
    }
}

fn resolve_kolme_live_signer_profile_selector_from_env() -> Result<Option<&'static str>, ConfigError>
{
    let profile_value = match env::var(KOLME_LIVE_SIGNER_PROFILE_ENV) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PROFILE_ENV} must be valid utf-8"
            )))
        }
    };
    let trimmed = profile_value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PROFILE_ENV} must not be empty"
        )));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(Some(KOLME_LIVE_SIGNER_PROFILE_PRIMARY)),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(Some(KOLME_LIVE_SIGNER_PROFILE_SECONDARY)),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PROFILE_ENV} has unsupported profile: {trimmed}"
        ))),
    }
}

pub(crate) fn resolve_kolme_live_signer_env_name_set(
    strict_signer_profile: Option<&str>,
) -> Result<(&'static str, &'static str, &'static str), ConfigError> {
    let profile_from_env = resolve_kolme_live_signer_profile_selector_from_env()?;
    let profile_value = if let Some(profile) = strict_signer_profile {
        let strict_profile = normalize_kolme_live_signer_profile_selector(profile)?;
        if let Some(env_profile) = profile_from_env {
            if env_profile != strict_profile {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "strict signer profile mismatch: --kolme-live-signer-profile={strict_profile} conflicts with {KOLME_LIVE_SIGNER_PROFILE_ENV}={env_profile} (runtime_signer_profile_selector_mismatch)"
                )));
            }
        }
        strict_profile
    } else {
        profile_from_env.unwrap_or(KOLME_LIVE_SIGNER_PROFILE_PRIMARY)
    };
    match profile_value {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok((
            KOLME_LIVE_SIGNER_PROFILE_PRIMARY,
            KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV,
            KOLME_LIVE_SIGNER_KEY_REF_ENV,
        )),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok((
            KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
            KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV,
            KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV,
        )),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "internal signer profile normalization invariant violated: {profile_value}"
        ))),
    }
}

pub(crate) fn read_required_kolme_live_key_reference_from_env(
    selection: &KolmeLiveSignerSelection,
) -> Result<String, ConfigError> {
    let key_reference = match env::var(selection.key_reference_env) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be set for signer profile {} when --kolme-live-signer-key-source={} (managed_signer_key_reference_missing)",
                selection.key_reference_env,
                selection.profile,
                KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
            )))
        }
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be valid utf-8 for signer profile {} (managed_signer_key_reference_invalid)",
                selection.key_reference_env, selection.profile
            )))
        }
    };
    normalize_kolme_live_managed_signer_key_reference(
        key_reference.as_str(),
        selection.profile,
        selection.key_reference_env,
    )
}

fn normalize_kolme_live_managed_signer_key_reference(
    value: &str,
    profile: &str,
    key_reference_env: &str,
) -> Result<String, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} must not be empty for signer profile {profile} (managed_signer_key_reference_invalid)"
        )));
    }
    SecureSignerProvider::from_key_id(trimmed).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} contains invalid secure key reference for signer profile {profile}: {error} (managed_signer_key_reference_invalid)"
        ))
    })?;
    let key_role = SignerKeyRole::from_key_id(trimmed).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} contains invalid signer role for signer profile {profile}: {error} (managed_signer_key_reference_invalid)"
        ))
    })?;
    if key_role != SignerKeyRole::Operator {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} must resolve to signer role operator for signer profile {profile}; found {} (managed_signer_key_reference_role_invalid)",
            key_role.label()
        )));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn resolve_kolme_live_signer_selection(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<KolmeLiveSignerSelection, ConfigError> {
    let key_source = if let Some(key_source) = strict_signer_key_source {
        normalize_kolme_live_signer_key_source(key_source)?
    } else if strict_signer_profile.is_some() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-key-source must be declared for strict signer contracts"
                .to_owned(),
        ));
    } else {
        KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL
    };
    let (profile, private_key_env, key_reference_env) =
        resolve_kolme_live_signer_env_name_set(strict_signer_profile)?;
    Ok(KolmeLiveSignerSelection {
        profile,
        key_source,
        private_key_env,
        key_reference_env,
    })
}

fn parse_kolme_live_signer_profile_value(
    value: &str,
    env_name: &str,
    reason_code: &str,
) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must not be empty when present ({reason_code})"
        )));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(KOLME_LIVE_SIGNER_PROFILE_PRIMARY),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(KOLME_LIVE_SIGNER_PROFILE_SECONDARY),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be one of {KOLME_LIVE_SIGNER_PROFILE_PRIMARY}, {KOLME_LIVE_SIGNER_PROFILE_SECONDARY}; found {trimmed} ({reason_code})"
        ))),
    }
}

fn parse_positive_u64_env_or_default(
    env_name: &str,
    default: u64,
    reason_code: &str,
) -> Result<u64, ConfigError> {
    match env::var(env_name) {
        Ok(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} must not be empty when present ({reason_code})"
                )));
            }
            let parsed = trimmed.parse::<u64>().map_err(|_| {
                ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} must be a positive integer, found '{trimmed}' ({reason_code})"
                ))
            })?;
            if parsed == 0 {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} must be greater than zero ({reason_code})"
                )));
            }
            Ok(parsed)
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be valid utf-8 when present ({reason_code})"
        ))),
    }
}

fn parse_positive_usize_env_or_default(
    env_name: &str,
    default: usize,
    reason_code: &str,
) -> Result<usize, ConfigError> {
    match env::var(env_name) {
        Ok(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} must not be empty when present ({reason_code})"
                )));
            }
            let parsed = trimmed.parse::<usize>().map_err(|_| {
                ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} must be a positive integer, found '{trimmed}' ({reason_code})"
                ))
            })?;
            if parsed == 0 {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{env_name} must be greater than zero ({reason_code})"
                )));
            }
            Ok(parsed)
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be valid utf-8 when present ({reason_code})"
        ))),
    }
}

fn resolve_kolme_live_signer_previous_profile(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<&'static str, ConfigError> {
    match env::var(KOLME_LIVE_SIGNER_PREVIOUS_PROFILE_ENV) {
        Ok(value) => parse_kolme_live_signer_profile_value(
            value.as_str(),
            KOLME_LIVE_SIGNER_PREVIOUS_PROFILE_ENV,
            "runtime_signer_previous_profile_invalid",
        ),
        Err(env::VarError::NotPresent) => Ok(signer_selection.profile),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PREVIOUS_PROFILE_ENV} must be valid utf-8 when present (runtime_signer_previous_profile_invalid)"
        ))),
    }
}

fn resolve_kolme_live_signer_quorum_approved_signers(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<Vec<&'static str>, ConfigError> {
    match env::var(KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV} must not be empty when present (runtime_signer_attestation_approved_signers_invalid)"
                )));
            }
            let mut seen = BTreeSet::new();
            let mut approved_signers = Vec::new();
            for entry in trimmed.split(',') {
                let profile = parse_kolme_live_signer_profile_value(
                    entry,
                    KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV,
                    "runtime_signer_attestation_approved_signers_invalid",
                )?;
                if !seen.insert(profile) {
                    return Err(ConfigError::RuntimeKolmeLive(format!(
                        "{KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV} must not contain duplicate signer profile {profile} (runtime_signer_attestation_approved_signers_not_unique)"
                    )));
                }
                approved_signers.push(profile);
            }
            if approved_signers.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV} must include at least one signer profile (runtime_signer_attestation_approved_signers_invalid)"
                )));
            }
            Ok(approved_signers)
        }
        Err(env::VarError::NotPresent) => Ok(vec![signer_selection.profile]),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS_ENV} must be valid utf-8 when present (runtime_signer_attestation_approved_signers_invalid)"
        ))),
    }
}

pub(crate) fn evaluate_kolme_live_signer_preflight_readiness(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<KolmeLiveSignerPreflightReadiness, ConfigError> {
    if signer_selection.profile == KOLME_LIVE_SIGNER_PROFILE_SECONDARY
        && signer_selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
    {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "signer profile {} cannot be paired with --kolme-live-signer-key-source={} (runtime_signer_key_source_profile_pair_disallowed)",
            signer_selection.profile, signer_selection.key_source
        )));
    }

    let previous_profile = resolve_kolme_live_signer_previous_profile(signer_selection)?;
    let failover_active = signer_selection.profile != previous_profile;
    let rotation_epoch = parse_positive_u64_env_or_default(
        KOLME_LIVE_SIGNER_ROTATION_EPOCH_ENV,
        1,
        "runtime_signer_rotation_epoch_invalid",
    )?;
    let previous_rotation_epoch = parse_positive_u64_env_or_default(
        KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH_ENV,
        1,
        "runtime_signer_previous_rotation_epoch_invalid",
    )?;
    match evaluate_signer_rotation_freshness(
        failover_active,
        rotation_epoch,
        previous_rotation_epoch,
    ) {
        SignerRotationFreshnessOutcome::Fresh => {}
        SignerRotationFreshnessOutcome::FailoverEpochStale {
            rotation_epoch,
            previous_rotation_epoch,
        } => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "signer failover rotation epoch must increase (current={rotation_epoch}, previous={previous_rotation_epoch}) (runtime_signer_rotation_epoch_stale)"
            )))
        }
        SignerRotationFreshnessOutcome::NonFailoverEpochRegressed {
            rotation_epoch,
            previous_rotation_epoch,
        } => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "signer rotation epoch must not regress when failover is inactive (current={rotation_epoch}, previous={previous_rotation_epoch}) (runtime_signer_rotation_epoch_regressed)"
            )))
        }
    }

    let approved_signers = resolve_kolme_live_signer_quorum_approved_signers(signer_selection)?;
    let quorum_required_approvals = parse_positive_usize_env_or_default(
        KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS_ENV,
        if failover_active { 2 } else { 1 },
        "runtime_signer_attestation_required_approvals_invalid",
    )?;
    if failover_active && quorum_required_approvals < 2 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS_ENV} must be at least 2 when signer failover is active (runtime_signer_failover_attestation_required_approvals_insufficient)"
        )));
    }

    if failover_active && !approved_signers.contains(&previous_profile) {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "approved signer set must include previous signer profile {previous_profile} during failover (runtime_signer_failover_attestation_previous_profile_not_approved)"
        )));
    }

    let quorum_approved_signers_count = approved_signers.len();
    let quorum_profile_linked = approved_signers.contains(&signer_selection.profile);
    let quorum_satisfied = quorum_approved_signers_count >= quorum_required_approvals;
    let quorum_linked = quorum_profile_linked && quorum_satisfied;

    if !quorum_profile_linked {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "current signer profile {} is not present in quorum-approved signer set (runtime_signer_quorum_linkage_violation)",
            signer_selection.profile
        )));
    }
    if !quorum_satisfied {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "runtime signer quorum shortfall: required {quorum_required_approvals}, approved {quorum_approved_signers_count} (runtime_signer_attestation_quorum_shortfall)"
        )));
    }

    Ok(KolmeLiveSignerPreflightReadiness {
        previous_profile,
        failover_active,
        rotation_epoch,
        previous_rotation_epoch,
        quorum_linkage_contract_version: KOLME_LIVE_SIGNER_QUORUM_LINKAGE_CONTRACT_VERSION,
        quorum_required_approvals,
        quorum_approved_signers_count,
        quorum_profile_linked,
        quorum_satisfied,
        quorum_linked,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_signer_rotation_freshness, SignerRotationFreshnessOutcome};

    #[test]
    fn unit_signer_rotation_freshness_outcome_matrix() {
        assert_eq!(
            evaluate_signer_rotation_freshness(false, 1, 1),
            SignerRotationFreshnessOutcome::Fresh
        );
        assert_eq!(
            evaluate_signer_rotation_freshness(false, 2, 1),
            SignerRotationFreshnessOutcome::Fresh
        );
        assert_eq!(
            evaluate_signer_rotation_freshness(true, 2, 1),
            SignerRotationFreshnessOutcome::Fresh
        );
        assert!(
            matches!(
                evaluate_signer_rotation_freshness(true, 1, 1),
                SignerRotationFreshnessOutcome::FailoverEpochStale {
                    rotation_epoch: 1,
                    previous_rotation_epoch: 1,
                }
            ),
            "failover stale epoch must emit typed stale outcome"
        );
        assert!(
            matches!(
                evaluate_signer_rotation_freshness(false, 1, 2),
                SignerRotationFreshnessOutcome::NonFailoverEpochRegressed {
                    rotation_epoch: 1,
                    previous_rotation_epoch: 2,
                }
            ),
            "non-failover epoch regression must emit typed regressed outcome"
        );
    }
}
