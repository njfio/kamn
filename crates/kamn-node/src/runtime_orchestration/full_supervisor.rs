use super::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(super) const FULL_SUPERVISOR_SERVICE_API_LANE_MAX_REQUESTS_CONTRACT_VIOLATION: &str =
    "full_supervisor_service_api_lane_max_requests_contract_violation";
pub(super) const FULL_SUPERVISOR_OBSERVABILITY_LANE_MAX_REQUESTS_CONTRACT_VIOLATION: &str =
    "full_supervisor_observability_lane_max_requests_contract_violation";
const FULL_SUPERVISOR_SERVICE_API_LANE_PROBE_FAILED: &str =
    "full_supervisor_service_api_lane_probe_failed";
const FULL_SUPERVISOR_OBSERVABILITY_LANE_PROBE_FAILED: &str =
    "full_supervisor_observability_lane_probe_failed";
const FULL_SUPERVISOR_SERVICE_API_LANE_EXECUTION_FAILED: &str =
    "full_supervisor_service_api_lane_execution_failed";
const FULL_SUPERVISOR_OBSERVABILITY_LANE_EXECUTION_FAILED: &str =
    "full_supervisor_observability_lane_execution_failed";
const FULL_SUPERVISOR_SERVICE_API_LANE_LIVENESS_FAILED: &str =
    "full_supervisor_service_api_lane_liveness_failed";
const FULL_SUPERVISOR_OBSERVABILITY_LANE_LIVENESS_FAILED: &str =
    "full_supervisor_observability_lane_liveness_failed";
const FULL_SUPERVISOR_SERVICE_API_LANE_JOIN_FAILED: &str =
    "full_supervisor_service_api_lane_join_failed";
const FULL_SUPERVISOR_OBSERVABILITY_LANE_JOIN_FAILED: &str =
    "full_supervisor_observability_lane_join_failed";
const FULL_SUPERVISOR_DAEMON_EXECUTION_JOIN_FAILED: &str =
    "full_supervisor_daemon_execution_join_failed";
const FULL_SUPERVISOR_LANE_IDLE_TIMEOUT_CONTENTION_GUARD_MS: u64 = 1_000;
const FULL_SUPERVISOR_PROVISIONAL_OBSERVABILITY_REASON_CODE: &str =
    "full_supervisor_bootstrap_in_progress";
const SERVICE_API_STATE_FILE_ENV: &str = "KAMN_SERVICE_API_STATE_FILE";
const SERVICE_API_RELAY_SPOOL_FILE_ENV: &str = "KAMN_SERVICE_API_RELAY_SPOOL_FILE";

pub(super) struct FullSupervisorServiceApiLane {
    config: ServiceApiEndpointConfig,
    handle: JoinHandle<Result<(), String>>,
}

pub(super) struct FullSupervisorObservabilityLane {
    config: ObservabilityEndpointConfig,
    handle: JoinHandle<Result<(), String>>,
}

fn full_supervisor_lane_error(reason_code: &'static str, detail: String) -> ConfigError {
    ConfigError::RuntimeDaemonLifecycle(format!(
        "full_supervisor_lane_failure:{reason_code}:{detail}"
    ))
}

fn is_service_api_idle_timeout_completion_error(error: &str) -> bool {
    error.starts_with("service api timed out after ") && error.ends_with(" ms waiting for requests")
}

fn is_observability_idle_timeout_completion_error(error: &str) -> bool {
    error.starts_with("observability endpoint timed out after ")
        && error.ends_with(" ms waiting for requests")
}

fn validate_full_supervisor_lane_liveness(
    service_api_lane: Option<&FullSupervisorServiceApiLane>,
    observability_lane: Option<&FullSupervisorObservabilityLane>,
) -> Result<(), ConfigError> {
    if let Some(lane) = service_api_lane {
        if lane.handle.is_finished() {
            return Err(full_supervisor_lane_error(
                FULL_SUPERVISOR_SERVICE_API_LANE_LIVENESS_FAILED,
                format!("bind_addr={}", lane.config.bind_addr),
            ));
        }
    }

    if let Some(lane) = observability_lane {
        if lane.handle.is_finished() {
            return Err(full_supervisor_lane_error(
                FULL_SUPERVISOR_OBSERVABILITY_LANE_LIVENESS_FAILED,
                format!("bind_addr={}", lane.config.bind_addr),
            ));
        }
    }

    Ok(())
}

pub(super) fn execute_full_supervisor_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
    service_api_lane: Option<&FullSupervisorServiceApiLane>,
    observability_lane: Option<&FullSupervisorObservabilityLane>,
) -> Result<DaemonExecution, ConfigError> {
    #[cfg(test)]
    let os_signal_test_triggers =
        super::daemon_shutdown::take_configured_os_signal_test_triggers_for_current_thread();
    let execution_id_owned = execution_id.to_owned();
    let daemon_handle = thread::spawn(move || {
        #[cfg(test)]
        configure_os_signal_test_triggers(os_signal_test_triggers);
        execute_daemon_runtime(runtime_mode, execution_id_owned.as_str(), options)
    });

    let mut service_api_inter_tick_probe_completed = service_api_lane.is_none();
    let mut observability_inter_tick_probe_completed = observability_lane.is_none();
    loop {
        if daemon_handle.is_finished() {
            break;
        }
        validate_full_supervisor_lane_liveness(service_api_lane, observability_lane)?;
        run_full_supervisor_inter_tick_lane_health_probes(
            service_api_lane.map(|lane| (lane.config.bind_addr.as_str(), "/healthz")),
            observability_lane.map(|lane| {
                (
                    lane.config.bind_addr.as_str(),
                    lane.config.health_path.as_str(),
                )
            }),
            &mut service_api_inter_tick_probe_completed,
            &mut observability_inter_tick_probe_completed,
        )?;
        thread::sleep(Duration::from_millis(1));
    }

    daemon_handle.join().map_err(|_| {
        ConfigError::RuntimeDaemonLifecycle(FULL_SUPERVISOR_DAEMON_EXECUTION_JOIN_FAILED.to_owned())
    })?
}

fn run_full_supervisor_inter_tick_lane_health_probes(
    service_api_probe_target: Option<(&str, &str)>,
    observability_probe_target: Option<(&str, &str)>,
    service_api_probe_completed: &mut bool,
    observability_probe_completed: &mut bool,
) -> Result<(), ConfigError> {
    if !*service_api_probe_completed {
        if let Some((bind_addr, path)) = service_api_probe_target {
            run_full_supervisor_http_probe(
                bind_addr,
                path,
                FULL_SUPERVISOR_SERVICE_API_LANE_PROBE_FAILED,
            )?;
        }
        *service_api_probe_completed = true;
    }

    if !*observability_probe_completed {
        if let Some((bind_addr, path)) = observability_probe_target {
            run_full_supervisor_http_probe(
                bind_addr,
                path,
                FULL_SUPERVISOR_OBSERVABILITY_LANE_PROBE_FAILED,
            )?;
        }
        *observability_probe_completed = true;
    }

    Ok(())
}

pub(super) fn full_supervisor_lane_idle_timeout_floor_ms(
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
) -> u64 {
    let expected_daemon_runtime_ms = daemon_max_ticks
        .unwrap_or(0)
        .saturating_mul(daemon_tick_interval_ms.unwrap_or(0));
    expected_daemon_runtime_ms.saturating_add(FULL_SUPERVISOR_LANE_IDLE_TIMEOUT_CONTENTION_GUARD_MS)
}

pub(super) fn request_full_supervisor_lane_shutdown_probes(
    service_api_lane: Option<&FullSupervisorServiceApiLane>,
    observability_lane: Option<&FullSupervisorObservabilityLane>,
) {
    if let Some(lane) = service_api_lane {
        let _ = run_full_supervisor_http_probe(
            lane.config.bind_addr.as_str(),
            "/healthz",
            FULL_SUPERVISOR_SERVICE_API_LANE_PROBE_FAILED,
        );
    }
    if let Some(lane) = observability_lane {
        let _ = run_full_supervisor_http_probe(
            lane.config.bind_addr.as_str(),
            lane.config.health_path.as_str(),
            FULL_SUPERVISOR_OBSERVABILITY_LANE_PROBE_FAILED,
        );
    }
}

pub(super) fn run_full_supervisor_http_probe(
    bind_addr: &str,
    path: &str,
    reason_code: &'static str,
) -> Result<(), ConfigError> {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        match TcpStream::connect(bind_addr) {
            Ok(mut stream) => {
                stream
                    .set_write_timeout(Some(Duration::from_millis(200)))
                    .map_err(|error| {
                        full_supervisor_lane_error(
                            reason_code,
                            format!("bind_addr={bind_addr};path={path};write_timeout:{error}"),
                        )
                    })?;
                stream
                    .set_read_timeout(Some(Duration::from_millis(200)))
                    .map_err(|error| {
                        full_supervisor_lane_error(
                            reason_code,
                            format!("bind_addr={bind_addr};path={path};read_timeout:{error}"),
                        )
                    })?;
                stream.write_all(request.as_bytes()).map_err(|error| {
                    full_supervisor_lane_error(
                        reason_code,
                        format!("bind_addr={bind_addr};path={path};write:{error}"),
                    )
                })?;
                let mut response_bytes = Vec::new();
                let mut buffer = [0_u8; 256];
                loop {
                    let read_count = stream.read(&mut buffer).map_err(|error| {
                        full_supervisor_lane_error(
                            reason_code,
                            format!("bind_addr={bind_addr};path={path};read:{error}"),
                        )
                    })?;
                    if read_count == 0 {
                        break;
                    }
                    response_bytes.extend_from_slice(&buffer[..read_count]);
                    if find_http_header_boundary(response_bytes.as_slice()).is_some()
                        || response_bytes.len() >= 1024
                    {
                        break;
                    }
                }
                if response_bytes.is_empty() {
                    return Err(full_supervisor_lane_error(
                        reason_code,
                        format!("bind_addr={bind_addr};path={path};empty_response"),
                    ));
                }
                let status_code =
                    parse_http_status_code(response_bytes.as_slice()).map_err(|detail| {
                        full_supervisor_lane_error(
                            reason_code,
                            format!("bind_addr={bind_addr};path={path};{detail}"),
                        )
                    })?;
                if !(200..300).contains(&status_code) {
                    return Err(full_supervisor_lane_error(
                        reason_code,
                        format!("bind_addr={bind_addr};path={path};http_status:{status_code}"),
                    ));
                }
                return Ok(());
            }
            Err(error) => {
                if Instant::now() >= deadline {
                    return Err(full_supervisor_lane_error(
                        reason_code,
                        format!("bind_addr={bind_addr};path={path};connect:{error}"),
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn find_http_header_boundary(response_bytes: &[u8]) -> Option<usize> {
    response_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

fn parse_http_status_code(response_bytes: &[u8]) -> Result<u16, String> {
    let header_end = find_http_header_boundary(response_bytes)
        .ok_or_else(|| "response_missing_header_terminator".to_owned())?;
    let header_text = std::str::from_utf8(&response_bytes[..header_end])
        .map_err(|_| "response_header_not_utf8".to_owned())?;
    let status_line = header_text
        .lines()
        .next()
        .ok_or_else(|| "response_missing_status_line".to_owned())?;
    let mut status_parts = status_line.split_whitespace();
    let _http_version = status_parts
        .next()
        .ok_or_else(|| "response_missing_http_version".to_owned())?;
    let raw_status = status_parts
        .next()
        .ok_or_else(|| "response_missing_status_code".to_owned())?;
    raw_status
        .parse::<u16>()
        .map_err(|_| format!("response_invalid_status_code:{raw_status}"))
}

pub(super) fn start_full_supervisor_service_api_lane(
    config: ServiceApiEndpointConfig,
    snapshot: super::service_api_endpoint::ServiceApiSnapshot,
    execution_id: &str,
) -> Result<FullSupervisorServiceApiLane, ConfigError> {
    let max_requests_label = config.max_requests.to_string();
    let idle_timeout_ms_label = config.idle_timeout_ms.to_string();
    let body_limit_bytes_label = config.body_limit_bytes.to_string();
    let concurrency_limit_label = config.concurrency_limit.to_string();
    let rate_limit_per_second_label = config.rate_limit_per_second.to_string();
    log_info(
        "node.runtime.service_api.endpoint.start",
        &[
            ("bind_addr", config.bind_addr.as_str()),
            ("max_requests", max_requests_label.as_str()),
            ("idle_timeout_ms", idle_timeout_ms_label.as_str()),
            ("body_limit_bytes", body_limit_bytes_label.as_str()),
            ("concurrency_limit", concurrency_limit_label.as_str()),
            (
                "rate_limit_per_second",
                rate_limit_per_second_label.as_str(),
            ),
            ("execution_id", execution_id),
        ],
    )?;

    let lane_config = config.clone();
    let lane_snapshot = snapshot.clone();
    let handle = thread::spawn(move || serve_service_api_endpoint(&lane_config, &lane_snapshot));
    if let Err(error) = run_full_supervisor_http_probe(
        config.bind_addr.as_str(),
        "/healthz",
        FULL_SUPERVISOR_SERVICE_API_LANE_PROBE_FAILED,
    ) {
        let _ = handle.join();
        return Err(error);
    }

    Ok(FullSupervisorServiceApiLane { config, handle })
}

pub(super) fn finish_full_supervisor_service_api_lane(
    lane: FullSupervisorServiceApiLane,
    execution_id: &str,
) -> Result<(), ConfigError> {
    let join_result = lane.handle.join().map_err(|_| {
        full_supervisor_lane_error(
            FULL_SUPERVISOR_SERVICE_API_LANE_JOIN_FAILED,
            format!("bind_addr={}", lane.config.bind_addr),
        )
    })?;
    if let Err(error) = join_result {
        if !is_service_api_idle_timeout_completion_error(error.as_str()) {
            return Err(full_supervisor_lane_error(
                FULL_SUPERVISOR_SERVICE_API_LANE_EXECUTION_FAILED,
                format!("bind_addr={};error={error}", lane.config.bind_addr),
            ));
        }
    }
    log_info(
        "node.runtime.service_api.endpoint.complete",
        &[
            ("bind_addr", lane.config.bind_addr.as_str()),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(())
}

pub(super) fn build_full_supervisor_provisional_observability_snapshot(
    runtime_mode: RuntimeMode,
) -> crate::observability_endpoint::RuntimeObservabilitySnapshot {
    crate::observability_endpoint::RuntimeObservabilitySnapshot {
        source: "daemon".to_owned(),
        runtime_mode: runtime_mode.as_str().to_owned(),
        latency_p50_ms: 0,
        latency_p99_ms: 0,
        throughput_tps: 0,
        error_rate_bps: 0,
        availability_bps: 0,
        health: "starting".to_owned(),
        alert_count: 0,
        reason_code: FULL_SUPERVISOR_PROVISIONAL_OBSERVABILITY_REASON_CODE.to_owned(),
        transport_checkpoint_failures: 0,
        signer_checkpoint_failures: 0,
        commit_checkpoint_failures: 0,
    }
}

pub(super) fn resolve_daemon_service_api_state_file(
    api_bind_addr: Option<&str>,
) -> Result<Option<String>, ConfigError> {
    match env::var(SERVICE_API_STATE_FILE_ENV) {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                    "{SERVICE_API_STATE_FILE_ENV} must not be empty when present"
                )));
            }
            Ok(Some(normalized.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(api_bind_addr
            .map(super::service_api_endpoint::default_service_api_state_file_path_for_bind_addr)),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_STATE_FILE_ENV} must be valid utf-8 when present"
        ))),
    }
}

pub(super) fn resolve_daemon_service_api_relay_spool_file(
    state_file: Option<&str>,
) -> Result<Option<String>, ConfigError> {
    match env::var(SERVICE_API_RELAY_SPOOL_FILE_ENV) {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                    "{SERVICE_API_RELAY_SPOOL_FILE_ENV} must not be empty when present"
                )));
            }
            Ok(Some(normalized.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(state_file.map(
            super::service_api_endpoint::default_service_api_relay_spool_file_path_from_state_file,
        )),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_SPOOL_FILE_ENV} must be valid utf-8 when present"
        ))),
    }
}

pub(super) fn start_full_supervisor_observability_lane(
    config: ObservabilityEndpointConfig,
    snapshot: crate::observability_endpoint::RuntimeObservabilitySnapshot,
    execution_id: &str,
) -> Result<FullSupervisorObservabilityLane, ConfigError> {
    let max_requests_label = config.max_requests.to_string();
    let idle_timeout_ms_label = config.idle_timeout_ms.to_string();
    log_info(
        "node.runtime.observability.endpoint.start",
        &[
            ("bind_addr", config.bind_addr.as_str()),
            ("metrics_path", config.metrics_path.as_str()),
            ("health_path", config.health_path.as_str()),
            ("max_requests", max_requests_label.as_str()),
            ("idle_timeout_ms", idle_timeout_ms_label.as_str()),
            ("execution_id", execution_id),
        ],
    )?;

    let lane_config = config.clone();
    let lane_snapshot = snapshot.clone();
    let handle = thread::spawn(move || serve_observability_endpoint(&lane_config, &lane_snapshot));
    if let Err(error) = run_full_supervisor_http_probe(
        config.bind_addr.as_str(),
        config.health_path.as_str(),
        FULL_SUPERVISOR_OBSERVABILITY_LANE_PROBE_FAILED,
    ) {
        let _ = handle.join();
        return Err(error);
    }

    Ok(FullSupervisorObservabilityLane { config, handle })
}

pub(super) fn finish_full_supervisor_observability_lane(
    lane: FullSupervisorObservabilityLane,
    execution_id: &str,
) -> Result<(), ConfigError> {
    let join_result = lane.handle.join().map_err(|_| {
        full_supervisor_lane_error(
            FULL_SUPERVISOR_OBSERVABILITY_LANE_JOIN_FAILED,
            format!("bind_addr={}", lane.config.bind_addr),
        )
    })?;
    if let Err(error) = join_result {
        if !is_observability_idle_timeout_completion_error(error.as_str()) {
            return Err(full_supervisor_lane_error(
                FULL_SUPERVISOR_OBSERVABILITY_LANE_EXECUTION_FAILED,
                format!("bind_addr={};error={error}", lane.config.bind_addr),
            ));
        }
    }
    log_info(
        "node.runtime.observability.endpoint.complete",
        &[
            ("bind_addr", lane.config.bind_addr.as_str()),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::thread;

    fn spawn_full_supervisor_probe_server(status_line: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let bind_addr = listener.local_addr().expect("local addr should resolve");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe connection should accept");
            let mut read_buffer = [0_u8; 1024];
            let _ = stream.read(&mut read_buffer);
            let response =
                format!("{status_line}\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
            stream
                .write_all(response.as_bytes())
                .expect("probe response should write");
        });
        bind_addr.to_string()
    }

    #[test]
    fn unit_full_supervisor_http_probe_accepts_success_status() {
        let bind_addr = spawn_full_supervisor_probe_server("HTTP/1.1 200 OK");
        run_full_supervisor_http_probe(
            bind_addr.as_str(),
            "/healthz",
            "full_supervisor_probe_success_test",
        )
        .expect("full supervisor probe should accept success status code");
    }

    #[test]
    fn regression_full_supervisor_http_probe_rejects_non_success_status() {
        // Regression: #5932
        let bind_addr = spawn_full_supervisor_probe_server("HTTP/1.1 503 Service Unavailable");
        let error = run_full_supervisor_http_probe(
            bind_addr.as_str(),
            "/healthz",
            "full_supervisor_probe_failure_test",
        )
        .expect_err("full supervisor probe must fail closed for non-success status");
        assert!(
            matches!(error, ConfigError::RuntimeDaemonLifecycle(ref message) if message.contains("http_status:503")),
            "probe failure should include deterministic http_status classification: {error:?}"
        );
    }

    #[test]
    fn unit_full_supervisor_inter_tick_probes_execute_once_per_lane() {
        let service_api_bind_addr = spawn_full_supervisor_probe_server("HTTP/1.1 200 OK");
        let observability_bind_addr = spawn_full_supervisor_probe_server("HTTP/1.1 200 OK");
        let mut service_api_probe_completed = false;
        let mut observability_probe_completed = false;

        run_full_supervisor_inter_tick_lane_health_probes(
            Some((service_api_bind_addr.as_str(), "/healthz")),
            Some((observability_bind_addr.as_str(), "/healthz")),
            &mut service_api_probe_completed,
            &mut observability_probe_completed,
        )
        .expect("inter-tick probes should succeed for healthy lanes");

        assert!(
            service_api_probe_completed,
            "service-api inter-tick probe should be marked completed after first success"
        );
        assert!(
            observability_probe_completed,
            "observability inter-tick probe should be marked completed after first success"
        );

        run_full_supervisor_inter_tick_lane_health_probes(
            Some((service_api_bind_addr.as_str(), "/healthz")),
            Some((observability_bind_addr.as_str(), "/healthz")),
            &mut service_api_probe_completed,
            &mut observability_probe_completed,
        )
        .expect("completed inter-tick probes should skip additional network probes");
    }

    #[test]
    fn regression_full_supervisor_inter_tick_probe_fails_closed_on_probe_error() {
        // Regression: #6143
        let service_api_bind_addr =
            spawn_full_supervisor_probe_server("HTTP/1.1 503 Service Unavailable");
        let mut service_api_probe_completed = false;
        let mut observability_probe_completed = true;

        let error = run_full_supervisor_inter_tick_lane_health_probes(
            Some((service_api_bind_addr.as_str(), "/healthz")),
            None,
            &mut service_api_probe_completed,
            &mut observability_probe_completed,
        )
        .expect_err("inter-tick probe must fail closed on non-success lane response");

        assert!(
                matches!(error, ConfigError::RuntimeDaemonLifecycle(ref message) if message.contains("http_status:503")),
                "inter-tick probe failure should preserve deterministic probe status classification: {error:?}"
            );
        assert!(
            !service_api_probe_completed,
            "failing inter-tick probe must not mark the service-api lane probe as completed"
        );
    }

    #[test]
    fn regression_full_supervisor_lane_idle_timeout_floor_adds_contention_guard_window() {
        // Regression: #6003
        assert_eq!(
            full_supervisor_lane_idle_timeout_floor_ms(Some(2), Some(10)),
            1_020
        );
        assert_eq!(
            full_supervisor_lane_idle_timeout_floor_ms(Some(0), Some(10)),
            1_000
        );
    }
}
