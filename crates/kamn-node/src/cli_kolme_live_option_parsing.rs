use super::ConfigError;

pub(super) struct KolmeLiveOptionState<'a> {
    pub(super) kolme_live_base_url: &'a mut Option<String>,
    pub(super) kolme_live_provider_hint: &'a mut Option<String>,
    pub(super) kolme_live_signing_profile: &'a mut Option<String>,
    pub(super) kolme_live_strict_signer_contracts: &'a mut bool,
    pub(super) kolme_live_signer_profile: &'a mut Option<String>,
    pub(super) kolme_live_signer_key_source: &'a mut Option<String>,
}

pub(super) fn try_parse_kolme_live_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut KolmeLiveOptionState<'_>,
) -> Result<bool, ConfigError> {
    if try_parse_kolme_live_base_options(arg, iter, state)? {
        return Ok(true);
    }
    try_parse_kolme_live_signer_options(arg, iter, state)
}

fn try_parse_kolme_live_base_options(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut KolmeLiveOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--kolme-live-base-url" => {
            set_string_option(iter, "--kolme-live-base-url", state.kolme_live_base_url)
                .map(|_| true)
        }
        "--kolme-live-provider-hint" => set_string_option(
            iter,
            "--kolme-live-provider-hint",
            state.kolme_live_provider_hint,
        )
        .map(|_| true),
        "--kolme-live-signing-profile" => set_string_option(
            iter,
            "--kolme-live-signing-profile",
            state.kolme_live_signing_profile,
        )
        .map(|_| true),
        "--kolme-live-strict-signer-contracts" => {
            *state.kolme_live_strict_signer_contracts = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn try_parse_kolme_live_signer_options(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut KolmeLiveOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--kolme-live-signer-profile" => set_string_option(
            iter,
            "--kolme-live-signer-profile",
            state.kolme_live_signer_profile,
        )
        .map(|_| true),
        "--kolme-live-signer-key-source" => set_string_option(
            iter,
            "--kolme-live-signer-key-source",
            state.kolme_live_signer_key_source,
        )
        .map(|_| true),
        _ => Ok(false),
    }
}

fn set_string_option(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut Option<String>,
) -> Result<(), ConfigError> {
    *target = Some(read_required_value(iter, flag)?);
    Ok(())
}

fn read_required_value(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<String, ConfigError> {
    iter.next().ok_or(ConfigError::MissingArgumentValue(flag))
}
