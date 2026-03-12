use std::env;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use kamn_core::{ConfigError, SigningRequest};

use super::{ManagedExternalBackendSignature, MANAGED_SIGNER_CHILD_ENV_ALLOWLIST};
use super::command::{
    parse_kolme_live_managed_signer_command_spec,
    resolve_kolme_live_managed_signer_timeout_seconds,
};
use super::response::parse_kolme_live_managed_signer_command_output;
use crate::KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS;

pub(super) fn execute_kolme_live_managed_signer_backend_command(
    command: &str,
    key_reference: &str,
    signing_request: &SigningRequest,
    canonical_message: &str,
) -> Result<ManagedExternalBackendSignature, ConfigError> {
    let timeout_seconds = resolve_kolme_live_managed_signer_timeout_seconds()?;
    let nonce = signing_request.nonce.to_string();
    let command_spec = parse_kolme_live_managed_signer_command_spec(command)?;
    let mut child_command = build_child_command(
        &command_spec,
        key_reference,
        signing_request,
        nonce.as_str(),
        canonical_message,
    );
    let mut child = child_command.spawn().map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend unavailable: failed to spawn command: {error} (managed_signer_backend_unavailable)"
        ))
    })?;
    let mut stdout = take_child_pipe(child.stdout.take(), "stdout")?;
    let mut stderr = take_child_pipe(child.stderr.take(), "stderr")?;
    let status = wait_for_child_completion(&mut child, timeout_seconds)?;
    let stdout_text = read_child_pipe(&mut stdout, "stdout")?;
    let stderr_text = read_child_pipe(&mut stderr, "stderr")?;
    ensure_success_status(status.success(), status.to_string().as_str(), stderr_text.as_str())?;
    parse_kolme_live_managed_signer_command_output(stdout_text.as_str())
}

fn build_child_command(
    command_spec: &super::ManagedSignerCommandSpec,
    key_reference: &str,
    signing_request: &SigningRequest,
    nonce: &str,
    canonical_message: &str,
) -> Command {
    let mut child_command = Command::new(command_spec.executable.as_str());
    child_command.args(command_spec.args.iter().map(String::as_str));
    child_command.env_clear();
    for env_name in MANAGED_SIGNER_CHILD_ENV_ALLOWLIST {
        if let Ok(value) = env::var(env_name) {
            child_command.env(env_name, value);
        }
    }
    child_command
        .env("KAMN_MANAGED_SIGNER_KEY_REFERENCE", key_reference)
        .env("KAMN_MANAGED_SIGNER_ACTOR_DID", signing_request.sender.as_str())
        .env("KAMN_MANAGED_SIGNER_NONCE", nonce)
        .env("KAMN_MANAGED_SIGNER_STATE_ROOT", signing_request.state_hash.as_str())
        .env("KAMN_MANAGED_SIGNER_CANONICAL_MESSAGE", canonical_message)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    child_command
}

fn take_child_pipe<T>(pipe: Option<T>, label: &str) -> Result<T, ConfigError> {
    pipe.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend unavailable: {label} pipe was not configured (managed_signer_backend_unavailable)"
        ))
    })
}

fn wait_for_child_completion(
    child: &mut std::process::Child,
    timeout_seconds: u64,
) -> Result<std::process::ExitStatus, ConfigError> {
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ConfigError::RuntimeKolmeLive(format!(
                        "managed-external signer backend timed out after {timeout_seconds}s (managed_signer_backend_timeout)"
                    )));
                }
                thread::sleep(Duration::from_millis(
                    KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS,
                ));
            }
            Err(error) => {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "managed-external signer backend unavailable while waiting for completion: {error} (managed_signer_backend_unavailable)"
                )))
            }
        }
    }
}

fn read_child_pipe(pipe: &mut impl Read, label: &str) -> Result<String, ConfigError> {
    let mut text = String::new();
    pipe.read_to_string(&mut text).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend unavailable: failed to read {label}: {error} (managed_signer_backend_unavailable)"
        ))
    })?;
    Ok(text)
}

fn ensure_success_status(success: bool, status: &str, stderr_text: &str) -> Result<(), ConfigError> {
    if success {
        return Ok(());
    }
    let stderr_trimmed = stderr_text.trim();
    let stderr_summary = if stderr_trimmed.is_empty() {
        "no stderr output"
    } else {
        stderr_trimmed
    };
    Err(ConfigError::RuntimeKolmeLive(format!(
        "managed-external signer backend unavailable: command exited with status {status} ({stderr_summary}) (managed_signer_backend_unavailable)"
    )))
}
