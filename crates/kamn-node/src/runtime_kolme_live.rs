use super::kolme_live_observability::build_kolme_live_observability_telemetry;
use super::signer::build_kolme_live_direct_signed_wire_payload;
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

fn deterministic_retry_backoff_millis(retry_attempt: u32) -> u64 {
    let exponent = retry_attempt.saturating_sub(1).min(4);
    let multiplier = 1_u64 << exponent;
    (KOLME_LIVE_RETRY_BASE_BACKOFF_MILLIS * multiplier).min(KOLME_LIVE_RETRY_MAX_BACKOFF_MILLIS)
}

fn maybe_sleep_retry_backoff(retry_attempt: u32) {
    let backoff = deterministic_retry_backoff_millis(retry_attempt);
    thread::sleep(Duration::from_millis(backoff));
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
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        provider_hint.as_str(),
        transport.clone(),
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let mut submit_attempts = 0_u32;
    let mut submit_retry_reason = "none";
    let submit_outcome = loop {
        submit_attempts += 1;
        match provider
            .submit_runtime_commit(signed_wire_payload.as_str(), request.idempotency_key())
        {
            Ok(outcome) => break outcome,
            Err(error) => {
                if let Some(reason_code) = classify_retry_category(&error) {
                    if submit_attempts < KOLME_LIVE_SUBMIT_RETRY_MAX_ATTEMPTS {
                        submit_retry_reason = reason_code;
                        maybe_sleep_retry_backoff(submit_attempts);
                        continue;
                    }
                    return Err(ConfigError::RuntimeKolmeLive(format!(
                        "submit retries exhausted after {submit_attempts} attempts ({reason_code}): {error}"
                    )));
                }
                return Err(ConfigError::RuntimeKolmeLive(error.to_string()));
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
                    if let Some(reason_code) = classify_retry_category(&error) {
                        if finality_retry_attempts < KOLME_LIVE_FINALITY_RETRY_MAX_ATTEMPTS {
                            finality_retry_reason = reason_code;
                            maybe_sleep_retry_backoff(finality_retry_attempts);
                            continue;
                        }
                        resolution = if reason_code == "timeout" {
                            "finality-timeout".to_owned()
                        } else {
                            "finality-unavailable".to_owned()
                        };
                        break;
                    }
                    if let KolmeRuntimeCommitProviderError::MalformedResponse { reason } = error {
                        return Err(ConfigError::RuntimeKolmeLive(format!(
                            "finality response malformed: {reason}"
                        )));
                    }
                    return Err(ConfigError::RuntimeKolmeLive(error.to_string()));
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
    let execution_status = format!(
        "{submit_status};commit_id={};finality={finality};resolution={resolution};submit_attempts={submit_attempts};submit_retry_reason={submit_retry_reason};finality_retry_attempts={finality_retry_attempts};finality_retry_reason={finality_retry_reason}",
        receipt.commit_id
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
    })
}

#[cfg(test)]
mod tests {
    use super::{
        classify_retry_category, deterministic_retry_backoff_millis, ConfigError,
        KolmeRuntimeCommitProviderError,
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
}
