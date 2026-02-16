use super::kolme_live_observability::build_kolme_live_observability_telemetry;
use super::signer::{
    build_kolme_live_direct_signed_wire_payload, evaluate_kolme_live_signer_preflight_readiness,
};
use super::{
    logging::{log_info, log_warn},
    KolmeLiveExecution, KOLME_IN_MEMORY_PROVIDER_MARKER, KOLME_LIVE_FINALITY_MAX_ATTEMPTS,
    KOLME_LIVE_FINALITY_STATUS_PATH, KOLME_LIVE_PROVIDER_CONTRACT, KOLME_LIVE_SIGNER_PROFILE_ENV,
    KOLME_LIVE_SIGNING_PROFILE, KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS,
};
use kamn_core::{
    BootstrapPlan, ConfigError, KolmeCommitReceiptFinality, KolmeRuntimeCommitFinalityChecker,
    KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitProvider,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitRequest,
};
use std::thread;
use std::time::Duration;

const KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS: u32 = 3;
const KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS: u32 = 3;
const KOLME_LIVE_RETRY_BASE_BACKOFF_MILLIS: u64 = 10;
const KOLME_LIVE_RETRY_MAX_BACKOFF_MILLIS: u64 = 40;

pub(crate) struct KolmeLiveContinuousMode {
    pub(crate) max_cycles: u64,
    pub(crate) cycle_interval_ms: u64,
}

pub(crate) fn build_kolme_live_request(
    plan: &BootstrapPlan,
) -> Result<KolmeRuntimeCommitRequest, ConfigError> {
    let role_label = plan.config.role.as_str();
    let operation_id = format!("runtime-commit:{}:{role_label}", plan.config.chain_id);
    let state_root = format!(
        "state:{}:{}",
        plan.config.chain_version, plan.state_schema.version.0
    );
    let actor_did = format!("kamn:did:agent:node-runtime-{role_label}");
    let payload_hash = format!("payload:{}:{role_label}", plan.config.chain_version);
    KolmeRuntimeCommitRequest::deterministic(
        operation_id.as_str(),
        state_root.as_str(),
        actor_did.as_str(),
        1,
        payload_hash.as_str(),
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))
}

pub(crate) fn ensure_kolme_live_provider_marker(
    expected: &str,
    observed: &str,
) -> Result<(), ConfigError> {
    if expected == observed {
        return Ok(());
    }
    Err(ConfigError::RuntimeKolmeLive(format!(
        "provider marker drift: expected '{expected}', observed '{observed}'"
    )))
}

pub(crate) fn kolme_live_finality_label(finality: KolmeCommitReceiptFinality) -> &'static str {
    match finality {
        KolmeCommitReceiptFinality::Pending => "pending",
        KolmeCommitReceiptFinality::Final => "final",
        KolmeCommitReceiptFinality::Failed => "failed",
    }
}

pub(crate) fn map_kolme_live_submit_outcome(
    outcome: KolmeRuntimeCommitProviderOutcome,
) -> Result<(&'static str, KolmeRuntimeCommitProviderReceipt), ConfigError> {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => Ok(("submitted", receipt)),
        KolmeRuntimeCommitProviderOutcome::Duplicate(receipt) => Ok(("duplicate", receipt)),
        KolmeRuntimeCommitProviderOutcome::Rejected { reason } => {
            Err(ConfigError::RuntimeKolmeLive(format!(
                "provider rejected runtime commit submission: {reason}"
            )))
        }
    }
}

fn classify_retry_category(error: &KolmeRuntimeCommitProviderError) -> Option<&'static str> {
    match error {
        KolmeRuntimeCommitProviderError::Timeout => Some("timeout"),
        KolmeRuntimeCommitProviderError::Unavailable { .. } => Some("unavailable"),
        KolmeRuntimeCommitProviderError::MalformedResponse { .. } => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetryDecision {
    Retry { reason_code: &'static str },
    Stop { reason_code: &'static str },
}

fn retry_decision_for_attempt(
    error: &KolmeRuntimeCommitProviderError,
    attempt: u32,
    max_attempts: u32,
) -> RetryDecision {
    match classify_retry_category(error) {
        Some(reason_code) if attempt < max_attempts => RetryDecision::Retry { reason_code },
        Some(_) => RetryDecision::Stop {
            reason_code: "attempt_ceiling_reached",
        },
        None => RetryDecision::Stop {
            reason_code: "malformed_response_fail_fast",
        },
    }
}

fn deterministic_retry_backoff_millis(retry_attempt: u32) -> u64 {
    let exponent = retry_attempt.saturating_sub(1).min(4);
    let multiplier = 1_u64 << exponent;
    (KOLME_LIVE_RETRY_BASE_BACKOFF_MILLIS * multiplier).min(KOLME_LIVE_RETRY_MAX_BACKOFF_MILLIS)
}

fn deterministic_retry_jitter_seed(correlation_id: &str) -> u64 {
    correlation_id
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |acc, byte| {
            (acc ^ u64::from(byte)).wrapping_mul(0x1000_0000_01b3)
        })
}

fn deterministic_retry_backoff_millis_with_jitter(retry_attempt: u32, jitter_seed: u64) -> u64 {
    let backoff = deterministic_retry_backoff_millis(retry_attempt);
    let rotate_by = retry_attempt.saturating_sub(1) % 63;
    let jitter = jitter_seed
        .rotate_left(rotate_by)
        .wrapping_add(u64::from(retry_attempt))
        % 4;
    backoff
        .saturating_add(jitter)
        .min(KOLME_LIVE_RETRY_MAX_BACKOFF_MILLIS)
}

pub(crate) fn execute_kolme_live_runtime(
    plan: &BootstrapPlan,
    base_url: String,
    provider_hint: String,
    signing_profile: String,
    strict_signer_profile: Option<&'static str>,
    strict_signer_key_source: Option<&'static str>,
) -> Result<KolmeLiveExecution, ConfigError> {
    if provider_hint.contains(KOLME_IN_MEMORY_PROVIDER_MARKER) {
        return Err(ConfigError::InvalidKolmeLiveProviderHint(provider_hint));
    }
    if signing_profile != KOLME_LIVE_SIGNING_PROFILE {
        return Err(ConfigError::InvalidKolmeLiveSigningProfile(signing_profile));
    }

    let mut transport = KolmeRuntimeCommitHttpTransport::new(KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let request = build_kolme_live_request(plan)?;
    let correlation_id = request.idempotency_key().to_owned();
    let retry_jitter_seed = deterministic_retry_jitter_seed(correlation_id.as_str());
    log_info(
        "kolme.live.submit.start",
        &[
            ("correlation_id", correlation_id.as_str()),
            ("provider_hint", provider_hint.as_str()),
            ("base_url", base_url.as_str()),
        ],
    )?;
    let (signed_wire_payload, signer_selection) = build_kolme_live_direct_signed_wire_payload(
        base_url.as_str(),
        &mut transport,
        &request,
        strict_signer_profile,
        strict_signer_key_source,
    )?;
    let signer_preflight = evaluate_kolme_live_signer_preflight_readiness(&signer_selection)?;
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        provider_hint.as_str(),
        transport.clone(),
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let mut submit_attempts = 0_u32;
    let mut submit_retry_reason = "none";
    let mut submit_retry_terminal_decision = "none";
    let submit_outcome = loop {
        submit_attempts += 1;
        match provider
            .submit_runtime_commit(signed_wire_payload.as_str(), request.idempotency_key())
        {
            Ok(outcome) => break outcome,
            Err(error) => {
                match retry_decision_for_attempt(
                    &error,
                    submit_attempts,
                    KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS,
                ) {
                    RetryDecision::Retry { reason_code } => {
                        submit_retry_reason = reason_code;
                        let attempt_label = submit_attempts.to_string();
                        let max_attempts_label = KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS.to_string();
                        let backoff_ms = deterministic_retry_backoff_millis_with_jitter(
                            submit_attempts,
                            retry_jitter_seed,
                        );
                        let backoff_ms_label = backoff_ms.to_string();
                        let jitter_seed_label = retry_jitter_seed.to_string();
                        log_warn(
                            "kolme.live.submit.retry",
                            &[
                                ("correlation_id", correlation_id.as_str()),
                                ("attempt", attempt_label.as_str()),
                                ("max_attempts", max_attempts_label.as_str()),
                                ("reason", reason_code),
                                ("decision", "retry"),
                                ("jitter_seed", jitter_seed_label.as_str()),
                                ("backoff_ms", backoff_ms_label.as_str()),
                            ],
                        )?;
                        thread::sleep(Duration::from_millis(backoff_ms));
                        continue;
                    }
                    RetryDecision::Stop {
                        reason_code: "attempt_ceiling_reached",
                    } => {
                        submit_retry_terminal_decision = "attempt_ceiling_reached";
                        let terminal_reason = classify_retry_category(&error).unwrap_or("unknown");
                        let attempt_label = submit_attempts.to_string();
                        let max_attempts_label = KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS.to_string();
                        log_warn(
                            "kolme.live.submit.retry.terminal",
                            &[
                                ("correlation_id", correlation_id.as_str()),
                                ("attempt", attempt_label.as_str()),
                                ("max_attempts", max_attempts_label.as_str()),
                                ("reason", terminal_reason),
                                ("decision", "stop"),
                                ("terminal_decision", submit_retry_terminal_decision),
                            ],
                        )?;
                        return Err(ConfigError::RuntimeKolmeLive(format!(
                            "submit retries exhausted after {submit_attempts} attempts ({terminal_reason}): {error}"
                        )));
                    }
                    RetryDecision::Stop {
                        reason_code: "malformed_response_fail_fast",
                    } => {
                        submit_retry_terminal_decision = "malformed_response_fail_fast";
                        let attempt_label = submit_attempts.to_string();
                        let max_attempts_label = KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS.to_string();
                        log_warn(
                            "kolme.live.submit.retry.terminal",
                            &[
                                ("correlation_id", correlation_id.as_str()),
                                ("attempt", attempt_label.as_str()),
                                ("max_attempts", max_attempts_label.as_str()),
                                ("reason", "malformed"),
                                ("decision", "fail-fast"),
                                ("terminal_decision", submit_retry_terminal_decision),
                            ],
                        )?;
                        return Err(ConfigError::RuntimeKolmeLive(error.to_string()));
                    }
                    RetryDecision::Stop { .. } => {
                        return Err(ConfigError::RuntimeKolmeLive(error.to_string()));
                    }
                }
            }
        }
    };
    let (submit_status, mut receipt) = map_kolme_live_submit_outcome(submit_outcome)?;
    ensure_kolme_live_provider_marker(provider_hint.as_str(), receipt.provider.as_str())?;
    log_info(
        "kolme.live.submit.outcome",
        &[
            ("correlation_id", correlation_id.as_str()),
            ("submit_status", submit_status),
            ("commit_id", receipt.commit_id.as_str()),
            ("finality", kolme_live_finality_label(receipt.finality)),
        ],
    )?;
    let mut resolution = "submit-receipt".to_owned();
    let mut finality_retry_attempts = 0_u32;
    let mut finality_retry_reason = "none";
    let mut finality_retry_terminal_decision = "none";

    if matches!(receipt.finality, KolmeCommitReceiptFinality::Pending) {
        log_info(
            "kolme.live.finality.poll.start",
            &[
                ("correlation_id", correlation_id.as_str()),
                ("commit_id", receipt.commit_id.as_str()),
            ],
        )?;
        let mut checker = KolmeRuntimeCommitFinalityChecker::new(
            base_url.as_str(),
            KOLME_LIVE_FINALITY_STATUS_PATH,
            transport,
        )
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
        loop {
            finality_retry_attempts += 1;
            match checker
                .poll_finality(receipt.commit_id.as_str(), KOLME_LIVE_FINALITY_MAX_ATTEMPTS)
            {
                Ok(polled_receipt) => {
                    ensure_kolme_live_provider_marker(
                        provider_hint.as_str(),
                        polled_receipt.provider.as_str(),
                    )?;
                    receipt = polled_receipt;
                    resolution = "finality-polled".to_owned();
                    break;
                }
                Err(error) => {
                    match retry_decision_for_attempt(
                        &error,
                        finality_retry_attempts,
                        KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS,
                    ) {
                        RetryDecision::Retry { reason_code } => {
                            finality_retry_reason = reason_code;
                            let attempt_label = finality_retry_attempts.to_string();
                            let max_attempts_label =
                                KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS.to_string();
                            let backoff_ms = deterministic_retry_backoff_millis_with_jitter(
                                finality_retry_attempts,
                                retry_jitter_seed,
                            );
                            let backoff_ms_label = backoff_ms.to_string();
                            let jitter_seed_label = retry_jitter_seed.to_string();
                            log_warn(
                                "kolme.live.finality.retry",
                                &[
                                    ("correlation_id", correlation_id.as_str()),
                                    ("commit_id", receipt.commit_id.as_str()),
                                    ("attempt", attempt_label.as_str()),
                                    ("max_attempts", max_attempts_label.as_str()),
                                    ("reason", reason_code),
                                    ("decision", "retry"),
                                    ("jitter_seed", jitter_seed_label.as_str()),
                                    ("backoff_ms", backoff_ms_label.as_str()),
                                ],
                            )?;
                            thread::sleep(Duration::from_millis(backoff_ms));
                            continue;
                        }
                        RetryDecision::Stop {
                            reason_code: "attempt_ceiling_reached",
                        } => {
                            finality_retry_terminal_decision = "attempt_ceiling_reached";
                            let terminal_reason =
                                classify_retry_category(&error).unwrap_or("unavailable");
                            let attempt_label = finality_retry_attempts.to_string();
                            let max_attempts_label =
                                KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS.to_string();
                            log_warn(
                                "kolme.live.finality.retry.terminal",
                                &[
                                    ("correlation_id", correlation_id.as_str()),
                                    ("commit_id", receipt.commit_id.as_str()),
                                    ("attempt", attempt_label.as_str()),
                                    ("max_attempts", max_attempts_label.as_str()),
                                    ("reason", terminal_reason),
                                    ("decision", "stop"),
                                    ("terminal_decision", finality_retry_terminal_decision),
                                ],
                            )?;
                            resolution = if terminal_reason == "timeout" {
                                "finality-timeout".to_owned()
                            } else {
                                "finality-unavailable".to_owned()
                            };
                            break;
                        }
                        RetryDecision::Stop {
                            reason_code: "malformed_response_fail_fast",
                        } => {
                            finality_retry_terminal_decision = "malformed_response_fail_fast";
                            let attempt_label = finality_retry_attempts.to_string();
                            let max_attempts_label =
                                KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS.to_string();
                            log_warn(
                                "kolme.live.finality.retry.terminal",
                                &[
                                    ("correlation_id", correlation_id.as_str()),
                                    ("commit_id", receipt.commit_id.as_str()),
                                    ("attempt", attempt_label.as_str()),
                                    ("max_attempts", max_attempts_label.as_str()),
                                    ("reason", "malformed"),
                                    ("decision", "fail-fast"),
                                    ("terminal_decision", finality_retry_terminal_decision),
                                ],
                            )?;
                            if let KolmeRuntimeCommitProviderError::MalformedResponse { reason } =
                                error
                            {
                                return Err(ConfigError::RuntimeKolmeLive(format!(
                                    "finality response malformed: {reason}"
                                )));
                            }
                            return Err(ConfigError::RuntimeKolmeLive(error.to_string()));
                        }
                        RetryDecision::Stop { .. } => {
                            return Err(ConfigError::RuntimeKolmeLive(error.to_string()));
                        }
                    }
                }
            }
        }
        if resolution == "finality-polled" {
            log_info(
                "kolme.live.finality.poll.outcome",
                &[
                    ("correlation_id", correlation_id.as_str()),
                    ("commit_id", receipt.commit_id.as_str()),
                    ("resolution", resolution.as_str()),
                    ("finality", kolme_live_finality_label(receipt.finality)),
                ],
            )?;
        } else {
            log_warn(
                "kolme.live.finality.poll.outcome",
                &[
                    ("correlation_id", correlation_id.as_str()),
                    ("commit_id", receipt.commit_id.as_str()),
                    ("resolution", resolution.as_str()),
                ],
            )?;
        }
    }

    let finality = kolme_live_finality_label(receipt.finality);
    let submit_retry_max_attempts = KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS;
    let finality_retry_max_attempts = KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS;
    let retry_backoff_base_ms = KOLME_LIVE_RETRY_BASE_BACKOFF_MILLIS;
    let retry_backoff_cap_ms = KOLME_LIVE_RETRY_MAX_BACKOFF_MILLIS;
    let retry_jitter_seed_label = retry_jitter_seed.to_string();
    let execution_status = format!(
        "{submit_status};commit_id={};finality={finality};resolution={resolution};submit_attempts={submit_attempts};submit_retry_reason={submit_retry_reason};submit_retry_terminal_decision={submit_retry_terminal_decision};submit_retry_max_attempts={submit_retry_max_attempts};finality_retry_attempts={finality_retry_attempts};finality_retry_reason={finality_retry_reason};finality_retry_terminal_decision={finality_retry_terminal_decision};finality_retry_max_attempts={finality_retry_max_attempts};retry_backoff_base_ms={retry_backoff_base_ms};retry_backoff_cap_ms={retry_backoff_cap_ms};retry_jitter_seed={retry_jitter_seed_label};signer_previous_profile={};signer_failover_active={};signer_rotation_epoch={};signer_previous_rotation_epoch={};signer_quorum_linkage_contract_version={};signer_quorum_required_approvals={};signer_quorum_approved_signers_count={};signer_quorum_profile_linked={};signer_quorum_satisfied={};signer_quorum_linked={}",
        receipt.commit_id
        ,
        signer_preflight.previous_profile,
        signer_preflight.failover_active,
        signer_preflight.rotation_epoch,
        signer_preflight.previous_rotation_epoch,
        signer_preflight.quorum_linkage_contract_version,
        signer_preflight.quorum_required_approvals,
        signer_preflight.quorum_approved_signers_count,
        signer_preflight.quorum_profile_linked,
        signer_preflight.quorum_satisfied,
        signer_preflight.quorum_linked
    );
    let observability = build_kolme_live_observability_telemetry(execution_status.as_str())
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    log_info(
        "kolme.live.execution.complete",
        &[
            ("correlation_id", correlation_id.as_str()),
            ("submit_status", submit_status),
            ("commit_id", receipt.commit_id.as_str()),
            ("finality", finality),
            ("resolution", resolution.as_str()),
        ],
    )?;
    Ok(KolmeLiveExecution {
        provider_client_contract: KOLME_LIVE_PROVIDER_CONTRACT.to_owned(),
        base_url,
        provider_hint,
        signing_profile,
        signer_profile_selector_env: KOLME_LIVE_SIGNER_PROFILE_ENV.to_owned(),
        signer_profile: signer_selection.profile.to_owned(),
        signer_key_source: signer_selection.key_source.to_owned(),
        signer_private_key_env: signer_selection.private_key_env.to_owned(),
        execution_status,
        observability_latency_p50_ms: observability.latency_p50_ms,
        observability_latency_p99_ms: observability.latency_p99_ms,
        observability_throughput_tps: observability.throughput_tps,
        observability_error_rate_bps: observability.error_rate_bps,
        observability_availability_bps: observability.availability_bps,
        observability_health: observability.health,
        observability_alert_count: observability.alert_count,
        observability_reason_code: observability.reason_code,
        observability_transport_checkpoint_failures: observability.transport_checkpoint_failures,
        observability_signer_checkpoint_failures: observability.signer_checkpoint_failures,
        observability_commit_checkpoint_failures: observability.commit_checkpoint_failures,
    })
}

pub(crate) fn execute_kolme_live_runtime_continuous(
    plan: &BootstrapPlan,
    base_url: String,
    provider_hint: String,
    signing_profile: String,
    strict_signer_profile: Option<&'static str>,
    strict_signer_key_source: Option<&'static str>,
    mode: KolmeLiveContinuousMode,
) -> Result<KolmeLiveExecution, ConfigError> {
    let max_cycles = mode.max_cycles;
    let cycle_interval_ms = mode.cycle_interval_ms;
    if max_cycles == 0 {
        return Err(ConfigError::RuntimeKolmeLive(
            "kolme-live continuous cycle budget must be greater than zero".to_owned(),
        ));
    }
    if cycle_interval_ms == 0 {
        return Err(ConfigError::RuntimeKolmeLive(
            "kolme-live continuous cycle interval must be greater than zero".to_owned(),
        ));
    }

    let mut last_execution: Option<KolmeLiveExecution> = None;
    for cycle in 1..=max_cycles {
        let mut execution = execute_kolme_live_runtime(
            plan,
            base_url.clone(),
            provider_hint.clone(),
            signing_profile.clone(),
            strict_signer_profile,
            strict_signer_key_source,
        )?;
        execution.execution_status = format!(
            "{};continuous_mode=enabled;continuous_cycle={cycle};continuous_cycle_count={max_cycles};continuous_cycle_interval_ms={cycle_interval_ms}",
            execution.execution_status
        );
        last_execution = Some(execution);
        if cycle < max_cycles {
            thread::sleep(Duration::from_millis(cycle_interval_ms));
        }
    }

    let mut execution = last_execution.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive("kolme-live continuous mode executed zero cycles".to_owned())
    })?;
    execution.execution_status = format!(
        "{};continuous_completed_cycles={max_cycles}",
        execution.execution_status
    );
    Ok(execution)
}

#[cfg(test)]
mod tests {
    use super::{
        classify_retry_category, deterministic_retry_backoff_millis,
        deterministic_retry_backoff_millis_with_jitter, deterministic_retry_jitter_seed,
        ensure_kolme_live_provider_marker, kolme_live_finality_label,
        map_kolme_live_submit_outcome, retry_decision_for_attempt, ConfigError,
        KolmeCommitReceiptFinality, KolmeRuntimeCommitProviderError,
        KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderReceipt, RetryDecision,
    };

    #[test]
    fn unit_retry_classifier_marks_transient_provider_errors() {
        assert_eq!(
            classify_retry_category(&KolmeRuntimeCommitProviderError::Timeout),
            Some("timeout")
        );
        assert_eq!(
            classify_retry_category(&KolmeRuntimeCommitProviderError::Unavailable {
                reason: "network unavailable".to_owned(),
            }),
            Some("unavailable")
        );
        assert_eq!(
            classify_retry_category(&KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "payload malformed".to_owned(),
            }),
            None
        );
    }

    #[test]
    fn unit_retry_backoff_policy_is_deterministic_and_bounded() {
        assert_eq!(deterministic_retry_backoff_millis(1), 10);
        assert_eq!(deterministic_retry_backoff_millis(2), 20);
        assert_eq!(deterministic_retry_backoff_millis(3), 40);
        assert_eq!(deterministic_retry_backoff_millis(8), 40);
    }

    #[test]
    fn unit_retry_decision_matrix_respects_attempt_ceiling_contract() {
        assert_eq!(
            retry_decision_for_attempt(&KolmeRuntimeCommitProviderError::Timeout, 1, 3),
            RetryDecision::Retry {
                reason_code: "timeout",
            }
        );
        assert_eq!(
            retry_decision_for_attempt(
                &KolmeRuntimeCommitProviderError::Unavailable {
                    reason: "temporary network fault".to_owned(),
                },
                2,
                3,
            ),
            RetryDecision::Retry {
                reason_code: "unavailable",
            }
        );
        assert_eq!(
            retry_decision_for_attempt(&KolmeRuntimeCommitProviderError::Timeout, 3, 3),
            RetryDecision::Stop {
                reason_code: "attempt_ceiling_reached",
            }
        );
        assert_eq!(
            retry_decision_for_attempt(
                &KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: "invalid json".to_owned(),
                },
                1,
                3,
            ),
            RetryDecision::Stop {
                reason_code: "malformed_response_fail_fast",
            }
        );
    }

    #[test]
    fn unit_retry_jitter_seed_contract_is_deterministic() {
        // Regression: #4109
        let correlation_id = "kolme:retry:seed:deterministic";
        assert_eq!(
            deterministic_retry_jitter_seed(correlation_id),
            deterministic_retry_jitter_seed(correlation_id)
        );
        assert_ne!(
            deterministic_retry_jitter_seed(correlation_id),
            deterministic_retry_jitter_seed("kolme:retry:seed:deterministic:alt")
        );
    }

    #[test]
    fn unit_retry_backoff_with_jitter_stays_bounded_and_deterministic() {
        let seed = deterministic_retry_jitter_seed("kolme:retry:jitter:bounded");
        assert_eq!(
            deterministic_retry_backoff_millis_with_jitter(1, seed),
            deterministic_retry_backoff_millis_with_jitter(1, seed)
        );
        assert_eq!(
            deterministic_retry_backoff_millis_with_jitter(2, seed),
            deterministic_retry_backoff_millis_with_jitter(2, seed)
        );
        assert!(
            deterministic_retry_backoff_millis_with_jitter(8, seed) <= 40,
            "retry backoff must remain capped at configured max"
        );
    }

    #[test]
    fn regression_retry_classifier_keeps_malformed_fail_fast() {
        // Regression: #2673
        assert_eq!(
            classify_retry_category(&KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "status field missing".to_owned(),
            }),
            None
        );
        let message =
            ConfigError::RuntimeKolmeLive("finality response malformed: x".to_owned()).to_string();
        assert!(
            message.contains("runtime kolme live validation failed"),
            "runtime malformed response errors should remain explicit and fail-fast"
        );
    }

    #[test]
    fn unit_kolme_live_finality_label_maps_all_variants() {
        assert_eq!(
            kolme_live_finality_label(KolmeCommitReceiptFinality::Pending),
            "pending"
        );
        assert_eq!(
            kolme_live_finality_label(KolmeCommitReceiptFinality::Final),
            "final"
        );
        assert_eq!(
            kolme_live_finality_label(KolmeCommitReceiptFinality::Failed),
            "failed"
        );
    }

    #[test]
    fn unit_kolme_live_provider_marker_guard_returns_deterministic_error() {
        let error = ensure_kolme_live_provider_marker("kolme-fork-local", "unexpected-provider")
            .expect_err("provider marker drift must fail closed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("provider marker drift")),
            "provider marker mismatch should return explicit runtime validation error"
        );
    }

    #[test]
    fn unit_kolme_live_submit_outcome_mapper_keeps_receipt_contract_and_rejection_reason() {
        let submitted_receipt = KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:abcd".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        };
        let duplicate_receipt = KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ef01".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        };
        let (submitted_status, submitted_mapped) = map_kolme_live_submit_outcome(
            KolmeRuntimeCommitProviderOutcome::Submitted(submitted_receipt.clone()),
        )
        .expect("submitted outcome should map");
        assert_eq!(submitted_status, "submitted");
        assert_eq!(submitted_mapped, submitted_receipt);

        let (duplicate_status, duplicate_mapped) = map_kolme_live_submit_outcome(
            KolmeRuntimeCommitProviderOutcome::Duplicate(duplicate_receipt.clone()),
        )
        .expect("duplicate outcome should map");
        assert_eq!(duplicate_status, "duplicate");
        assert_eq!(duplicate_mapped, duplicate_receipt);

        let rejected_error =
            map_kolme_live_submit_outcome(KolmeRuntimeCommitProviderOutcome::Rejected {
                reason: "policy-denied".to_owned(),
            })
            .expect_err("rejected outcome must fail closed");
        assert!(
            matches!(rejected_error, ConfigError::RuntimeKolmeLive(message) if message.contains("policy-denied")),
            "rejected outcome must preserve rejection reason for localized debugging"
        );
    }
}
