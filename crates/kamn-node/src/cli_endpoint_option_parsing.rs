use super::{cli_value_parsers::parse_daemon_control_arg, ConfigError};

pub(super) struct EndpointOptionState<'a> {
    pub(super) api_bind_addr: &'a mut Option<String>,
    pub(super) api_max_requests: &'a mut u64,
    pub(super) api_idle_timeout_ms: &'a mut u64,
    pub(super) api_body_limit_bytes: &'a mut u64,
    pub(super) api_concurrency_limit: &'a mut u64,
    pub(super) api_rate_limit_per_second: &'a mut u64,
    pub(super) observability_endpoint_bind_addr: &'a mut Option<String>,
    pub(super) observability_endpoint_metrics_path: &'a mut String,
    pub(super) observability_endpoint_health_path: &'a mut String,
    pub(super) observability_endpoint_max_requests: &'a mut u64,
    pub(super) observability_endpoint_idle_timeout_ms: &'a mut u64,
    pub(super) api_max_requests_overridden: &'a mut bool,
    pub(super) api_idle_timeout_ms_overridden: &'a mut bool,
    pub(super) api_body_limit_bytes_overridden: &'a mut bool,
    pub(super) api_concurrency_limit_overridden: &'a mut bool,
    pub(super) api_rate_limit_per_second_overridden: &'a mut bool,
    pub(super) observability_endpoint_metrics_path_overridden: &'a mut bool,
    pub(super) observability_endpoint_health_path_overridden: &'a mut bool,
    pub(super) observability_endpoint_max_requests_overridden: &'a mut bool,
    pub(super) observability_endpoint_idle_timeout_ms_overridden: &'a mut bool,
}

pub(super) fn try_parse_endpoint_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut EndpointOptionState<'_>,
) -> Result<bool, ConfigError> {
    if try_parse_api_endpoint_option(arg, iter, state)? {
        return Ok(true);
    }
    try_parse_observability_endpoint_option(arg, iter, state)
}

fn try_parse_api_endpoint_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut EndpointOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--api-bind" => parse_bind_option(iter, "--api-bind", state.api_bind_addr).map(|_| true),
        "--api-max-requests" => set_numeric_option(
            iter,
            "--api-max-requests",
            state.api_max_requests,
            state.api_max_requests_overridden,
        )
        .map(|_| true),
        "--api-idle-timeout-ms" => set_numeric_option(
            iter,
            "--api-idle-timeout-ms",
            state.api_idle_timeout_ms,
            state.api_idle_timeout_ms_overridden,
        )
        .map(|_| true),
        "--api-body-limit-bytes" => set_numeric_option(
            iter,
            "--api-body-limit-bytes",
            state.api_body_limit_bytes,
            state.api_body_limit_bytes_overridden,
        )
        .map(|_| true),
        "--api-concurrency-limit" => set_numeric_option(
            iter,
            "--api-concurrency-limit",
            state.api_concurrency_limit,
            state.api_concurrency_limit_overridden,
        )
        .map(|_| true),
        "--api-rate-limit-per-second" => set_numeric_option(
            iter,
            "--api-rate-limit-per-second",
            state.api_rate_limit_per_second,
            state.api_rate_limit_per_second_overridden,
        )
        .map(|_| true),
        _ => Ok(false),
    }
}

fn try_parse_observability_endpoint_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut EndpointOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--observability-endpoint-bind" => parse_bind_option(
            iter,
            "--observability-endpoint-bind",
            state.observability_endpoint_bind_addr,
        )
        .map(|_| true),
        "--observability-endpoint-metrics-path" => set_string_option(
            iter,
            "--observability-endpoint-metrics-path",
            state.observability_endpoint_metrics_path,
            state.observability_endpoint_metrics_path_overridden,
        )
        .map(|_| true),
        "--observability-endpoint-health-path" => set_string_option(
            iter,
            "--observability-endpoint-health-path",
            state.observability_endpoint_health_path,
            state.observability_endpoint_health_path_overridden,
        )
        .map(|_| true),
        "--observability-endpoint-max-requests" => set_numeric_option(
            iter,
            "--observability-endpoint-max-requests",
            state.observability_endpoint_max_requests,
            state.observability_endpoint_max_requests_overridden,
        )
        .map(|_| true),
        "--observability-endpoint-idle-timeout-ms" => set_numeric_option(
            iter,
            "--observability-endpoint-idle-timeout-ms",
            state.observability_endpoint_idle_timeout_ms,
            state.observability_endpoint_idle_timeout_ms_overridden,
        )
        .map(|_| true),
        _ => Ok(false),
    }
}

fn parse_bind_option(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut Option<String>,
) -> Result<(), ConfigError> {
    *target = Some(read_required_value(iter, flag)?);
    Ok(())
}

fn set_numeric_option(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut u64,
    overridden: &mut bool,
) -> Result<(), ConfigError> {
    *target = parse_numeric_flag(iter, flag)?;
    *overridden = true;
    Ok(())
}

fn set_string_option(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut String,
    overridden: &mut bool,
) -> Result<(), ConfigError> {
    *target = read_required_value(iter, flag)?;
    *overridden = true;
    Ok(())
}

fn read_required_value(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<String, ConfigError> {
    iter.next().ok_or(ConfigError::MissingArgumentValue(flag))
}

fn parse_numeric_flag(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<u64, ConfigError> {
    let value = read_required_value(iter, flag)?;
    parse_daemon_control_arg(&value)
}
