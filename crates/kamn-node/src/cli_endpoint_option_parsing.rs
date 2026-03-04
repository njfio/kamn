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
        "--api-bind" => {
            *state.api_bind_addr = Some(read_required_value(iter, "--api-bind")?);
            Ok(true)
        }
        "--api-max-requests" => {
            *state.api_max_requests = parse_numeric_flag(iter, "--api-max-requests")?;
            *state.api_max_requests_overridden = true;
            Ok(true)
        }
        "--api-idle-timeout-ms" => {
            *state.api_idle_timeout_ms = parse_numeric_flag(iter, "--api-idle-timeout-ms")?;
            *state.api_idle_timeout_ms_overridden = true;
            Ok(true)
        }
        "--api-body-limit-bytes" => {
            *state.api_body_limit_bytes = parse_numeric_flag(iter, "--api-body-limit-bytes")?;
            *state.api_body_limit_bytes_overridden = true;
            Ok(true)
        }
        "--api-concurrency-limit" => {
            *state.api_concurrency_limit = parse_numeric_flag(iter, "--api-concurrency-limit")?;
            *state.api_concurrency_limit_overridden = true;
            Ok(true)
        }
        "--api-rate-limit-per-second" => {
            *state.api_rate_limit_per_second =
                parse_numeric_flag(iter, "--api-rate-limit-per-second")?;
            *state.api_rate_limit_per_second_overridden = true;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn try_parse_observability_endpoint_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut EndpointOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--observability-endpoint-bind" => {
            *state.observability_endpoint_bind_addr =
                Some(read_required_value(iter, "--observability-endpoint-bind")?);
            Ok(true)
        }
        "--observability-endpoint-metrics-path" => {
            *state.observability_endpoint_metrics_path =
                read_required_value(iter, "--observability-endpoint-metrics-path")?;
            *state.observability_endpoint_metrics_path_overridden = true;
            Ok(true)
        }
        "--observability-endpoint-health-path" => {
            *state.observability_endpoint_health_path =
                read_required_value(iter, "--observability-endpoint-health-path")?;
            *state.observability_endpoint_health_path_overridden = true;
            Ok(true)
        }
        "--observability-endpoint-max-requests" => {
            *state.observability_endpoint_max_requests =
                parse_numeric_flag(iter, "--observability-endpoint-max-requests")?;
            *state.observability_endpoint_max_requests_overridden = true;
            Ok(true)
        }
        "--observability-endpoint-idle-timeout-ms" => {
            *state.observability_endpoint_idle_timeout_ms =
                parse_numeric_flag(iter, "--observability-endpoint-idle-timeout-ms")?;
            *state.observability_endpoint_idle_timeout_ms_overridden = true;
            Ok(true)
        }
        _ => Ok(false),
    }
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
