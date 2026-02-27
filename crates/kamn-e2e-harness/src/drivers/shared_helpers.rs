use std::env;

const S07_REPLAY_REASON_MARKER: &str = "service_api_auth_replay_nonce_detected";

pub(crate) fn env_var_or_default(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(value) => value,
        Err(_) => default.to_owned(),
    }
}

pub(crate) fn env_var_or_else<F>(key: &str, fallback: F) -> String
where
    F: FnOnce() -> String,
{
    match env::var(key) {
        Ok(value) => value,
        Err(_) => fallback(),
    }
}

pub(crate) fn is_live_bound_scenario_id(scenario_id: &str) -> bool {
    matches!(
        scenario_id,
        "S-01"
            | "S-02"
            | "S-03"
            | "S-04"
            | "S-05"
            | "S-06"
            | "S-07"
            | "S-08"
            | "S-09"
            | "S-10"
            | "S-11"
            | "S-12"
            | "S-13"
            | "S-14"
            | "S-15"
    )
}

pub(crate) fn live_execution_enabled_from_env(live_env_key: &str) -> bool {
    env::var(live_env_key)
        .ok()
        .map(|value| parse_bool_flag(value.as_str()))
        .unwrap_or(false)
}

pub(crate) fn parse_bool_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub(crate) fn validate_live_s05_release_escrow_response(
    expected_escrow_id: &str,
    released_escrow_id: &str,
    release_state: &str,
    step: &str,
) -> Result<(), String> {
    if released_escrow_id != expected_escrow_id {
        return Err(format!(
            "{step} returned mismatched escrow_id: expected={expected_escrow_id}, got={released_escrow_id}"
        ));
    }
    if release_state.trim().is_empty() {
        return Err(format!("{step} returned empty state"));
    }
    Ok(())
}

pub(crate) fn validate_s12_content_id_match(
    expected_content_id: &str,
    observed_content_id: &str,
    step: &str,
) -> Result<(), String> {
    if observed_content_id != expected_content_id {
        return Err(format!(
            "{step} returned mismatched content_id: expected={expected_content_id}, got={observed_content_id}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_s12_content_field_coherence(
    expected_field_value: &str,
    observed_field_value: &str,
    field_name: &str,
    step: &str,
) -> Result<(), String> {
    if observed_field_value != expected_field_value {
        return Err(format!(
            "{step} {field_name} drift: expected={expected_field_value}, got={observed_field_value}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_s13_bridge_id_match(
    expected_bridge_id: &str,
    observed_bridge_id: &str,
    step: &str,
) -> Result<(), String> {
    if observed_bridge_id != expected_bridge_id {
        return Err(format!(
            "{step} returned mismatched bridge_id: expected={expected_bridge_id}, got={observed_bridge_id}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_s13_bridge_field_coherence(
    expected_field_value: &str,
    observed_field_value: &str,
    field_name: &str,
    step: &str,
) -> Result<(), String> {
    if observed_field_value != expected_field_value {
        return Err(format!(
            "{step} {field_name} drift: expected={expected_field_value}, got={observed_field_value}"
        ));
    }
    Ok(())
}

pub(crate) fn parse_s15_budget_env_u128(
    env_key: &str,
    default_value: u128,
    step: &str,
) -> Result<u128, String> {
    let parsed = env::var(env_key)
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u128>()
                .map_err(|_| format!("{step} invalid env value for {env_key}: {raw}"))
        })
        .transpose()?
        .unwrap_or(default_value);
    if parsed == 0 {
        return Err(format!("{step} must be greater than zero for {env_key}"));
    }
    Ok(parsed)
}

pub(crate) fn validate_s15_latency_budget_samples(
    samples_millis: &[u128],
    total_elapsed_millis: u128,
    max_total_millis: u128,
    max_p50_millis: u128,
    max_p99_millis: u128,
    step: &str,
) -> Result<(), String> {
    if samples_millis.is_empty() {
        return Err(format!("{step} produced zero latency samples"));
    }

    let mut sorted = samples_millis.to_vec();
    sorted.sort_unstable();
    let p50_index = percentile_index(sorted.len(), 50);
    let p99_index = percentile_index(sorted.len(), 99);
    let p50 = sorted[p50_index];
    let p99 = sorted[p99_index];

    if total_elapsed_millis > max_total_millis {
        return Err(format!(
            "{step} total elapsed millis exceeded budget: observed={total_elapsed_millis}, max={max_total_millis}"
        ));
    }
    if p50 > max_p50_millis {
        return Err(format!(
            "{step} p50 millis exceeded budget: observed={p50}, max={max_p50_millis}"
        ));
    }
    if p99 > max_p99_millis {
        return Err(format!(
            "{step} p99 millis exceeded budget: observed={p99}, max={max_p99_millis}"
        ));
    }
    Ok(())
}

pub(crate) fn percentile_index(sample_count: usize, percentile: u128) -> usize {
    let numerator = (sample_count as u128)
        .saturating_mul(percentile)
        .saturating_add(100u128.saturating_sub(1));
    let rank = numerator / 100;
    rank.saturating_sub(1).min(sample_count as u128 - 1) as usize
}

pub(crate) fn validate_s07_replay_reason_marker(
    replay_error: &str,
    step: &str,
) -> Result<(), String> {
    if !replay_error.contains(S07_REPLAY_REASON_MARKER) {
        return Err(format!(
            "{step} missing replay reason marker: {replay_error}"
        ));
    }
    Ok(())
}

pub(crate) fn live_s07_probe_agent_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}
