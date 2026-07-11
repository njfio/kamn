use std::path::Path;

use super::live_task_binding::LiveTaskBinding;
use kamn_agent_lib::{AgentMetadata, KamnAgentHandle};

mod agreement;

use agreement::SettlementAgreement;

const SDK_TIMEOUT_ENV: &str = "KAMN_SDK_SERVICE_TIMEOUT_SECONDS";
const LIVE_SETTLEMENT_TIMEOUT_SECONDS: &str = "90";
pub(super) const CREATOR_AGENT_NAME: &str = "kamn-mvp-devnet-settlement-creator";

pub(super) fn drive_escrow_release(
    endpoint: &str,
    run_dir: &Path,
    binding: Option<&LiveTaskBinding>,
    amount_lamports: u64,
) -> Result<SettlementRun, String> {
    let _timeout = EnvOverride::set(SDK_TIMEOUT_ENV, LIVE_SETTLEMENT_TIMEOUT_SECONDS);
    let creator = agent(endpoint, CREATOR_AGENT_NAME)?;
    let provider = agent(endpoint, "kamn-mvp-devnet-settlement-provider")?;
    register(&creator, "creator")?;
    register(&provider, "provider")?;
    let agreement =
        SettlementAgreement::new(run_dir, binding, amount_lamports, &creator, &provider)?;
    let task = creator
        .create_task(agreement.task_payload().as_str())
        .map_err(|error| format!("failed to create MVP demo task: {error}"))?;
    provider
        .accept_task_with_payload(task.task_id.as_str(), agreement.accept_payload().as_str())
        .map_err(|error| format!("failed to accept MVP demo task: {error}"))?;
    let payload = agreement.fund_payload(task.task_id.as_str());
    write_funding_payload(run_dir, payload.as_str())?;
    let funded = creator
        .fund_escrow(payload.as_str())
        .map_err(|error| format!("failed to fund MVP demo escrow: {error}"))?;
    require_expected_escrow_id(payload.as_str(), funded.escrow_id.as_str())?;
    provider
        .complete_task_with_payload(task.task_id.as_str(), agreement.complete_payload().as_str())
        .map_err(|error| format!("failed to complete MVP demo task: {error}"))?;
    std::thread::sleep(release_pacing_delay());
    let released = release_with_reconciliation(
        &creator,
        funded.escrow_id.as_str(),
        agreement.release_payload().as_str(),
    )?;
    require_released(released.state.as_str())?;
    Ok(SettlementRun {
        escrow_id: released.escrow_id,
        task_id: task.task_id,
    })
}

fn release_pacing_delay() -> std::time::Duration {
    std::time::Duration::from_millis(5_100)
}

fn release_with_reconciliation(
    creator: &KamnAgentHandle,
    escrow_id: &str,
    payload: &str,
) -> Result<kamn_agent_lib::ServiceEscrowStatus, String> {
    let attempts = release_attempt_limit();
    for attempt in 0..attempts {
        match creator.release_escrow_with_payload(escrow_id, payload) {
            Ok(released) => return Ok(released),
            Err(error)
                if attempt + 1 < attempts && should_retry_release(error.to_string().as_str()) =>
            {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(error) => return Err(format!("failed to release MVP demo escrow: {error}")),
        }
    }
    Err("failed to reconcile MVP demo escrow release".to_owned())
}

fn release_attempt_limit() -> usize {
    15
}

fn should_retry_release(error: &str) -> bool {
    error.contains("SETTLEMENT_OUTCOME_AMBIGUOUS")
        || error.contains("live settlement evidence failed")
}

pub(super) struct SettlementRun {
    pub(super) escrow_id: String,
    pub(super) task_id: String,
}

fn agent(endpoint: &str, name: &str) -> Result<KamnAgentHandle, String> {
    KamnAgentHandle::connect(endpoint, "http://127.0.0.1:13000", name)
        .map_err(|error| format!("failed to create KAMN agent handle: {error}"))
}

fn register(handle: &KamnAgentHandle, role: &str) -> Result<(), String> {
    let metadata = AgentMetadata {
        agent_type: format!("mvp-settlement-{role}"),
        model_family: "deterministic-e2e-harness".to_owned(),
        capabilities: vec!["task-settlement".to_owned()],
    };
    handle
        .register_agent(&metadata)
        .map(|_| ())
        .map_err(|error| format!("failed to register MVP demo {role}: {error}"))
}

fn require_released(state: &str) -> Result<(), String> {
    if state == "released" {
        return Ok(());
    }
    Err(format!(
        "devnet settlement escrow state not released: {state}"
    ))
}

struct EnvOverride {
    name: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvOverride {
    fn set(name: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(name);
        std::env::set_var(name, value);
        Self { name, previous }
    }
}

impl Drop for EnvOverride {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.name, value),
            None => std::env::remove_var(self.name),
        }
    }
}

fn write_funding_payload(run_dir: &Path, payload: &str) -> Result<(), String> {
    std::fs::write(
        run_dir.join("proof/devnet-escrow-funding-request.json"),
        payload,
    )
    .map_err(|error| format!("failed to write devnet escrow funding request: {error}"))
}

fn require_expected_escrow_id(payload: &str, escrow_id: &str) -> Result<(), String> {
    let expected = expected_escrow_id(payload);
    if escrow_id == expected {
        Ok(())
    } else {
        Err(format!(
            "devnet settlement escrow ID mismatch: expected {expected}, found {escrow_id}"
        ))
    }
}

pub(super) fn expected_escrow_id(payload: &str) -> String {
    format!(
        "escrow-local-{:016x}",
        deterministic_body_tag(payload.as_bytes())
    )
}

fn deterministic_body_tag(payload: &[u8]) -> u64 {
    payload.iter().fold(0xcbf29ce484222325_u64, |acc, byte| {
        acc.wrapping_mul(0x00000100000001B3) ^ u64::from(*byte)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn funded_rehearsal_payload_carries_canonical_task_agreement() {
        let run_dir = Path::new("/tmp/run-contract");
        let binding = LiveTaskBinding {
            artifact_path: "binding.json".to_owned(),
            digest: "a".repeat(64),
            task_id: "task-external".to_owned(),
            agent_a_pid: 1,
            agent_b_pid: 2,
            agent_c_pid: 3,
        };
        let creator = agent("http://127.0.0.1:1", "contract-creator").expect("creator");
        let provider = agent("http://127.0.0.1:1", "contract-provider").expect("provider");
        let agreement =
            SettlementAgreement::new(run_dir, Some(&binding), 1_000_000, &creator, &provider)
                .expect("agreement");
        let raw = agreement.fund_payload("task-settlement");
        for field in [
            "task_id",
            "transaction_id",
            "beneficiary_did",
            "amount_lamports",
            "network",
            "terms_digest",
            "release_authority_did",
            "release_policy",
            "idempotency_key",
        ] {
            assert!(
                raw.contains(format!("\"{field}\":").as_str()),
                "missing canonical field {field}"
            );
        }
        assert!(raw.contains(r#""network":"solana-devnet""#));
        assert!(raw.contains(r#""release_policy":"task-completed""#));
    }

    #[test]
    fn funded_rehearsal_paces_release_past_sender_window() {
        assert!(release_pacing_delay() > std::time::Duration::from_secs(5));
    }

    #[test]
    fn funded_rehearsal_retries_only_recoverable_settlement_visibility() {
        assert_eq!(release_attempt_limit(), 15);
        assert!(should_retry_release(
            "service api live settlement evidence failed: confirmation missing"
        ));
        assert!(should_retry_release("SETTLEMENT_OUTCOME_AMBIGUOUS"));
        assert!(!should_retry_release("ACTION_NOT_GRANTED"));
        assert!(!should_retry_release("SETTLEMENT_INTENT_CONFLICT"));
    }
}
