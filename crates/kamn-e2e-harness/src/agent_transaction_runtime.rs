use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use super::agent_transaction_process::terminate_process;
use super::AgentTransactionDemoConfig;

const RUNTIME_ERROR: &str = "AGENT_TRANSACTION_RUNTIME_FAILED";

pub(super) struct LocalRuntime {
    child: Child,
}

impl LocalRuntime {
    pub(super) fn start(config: &AgentTransactionDemoConfig) -> Result<Self, String> {
        let address = endpoint_address(config.mcp_endpoint.as_str())?;
        let mut command = runtime_command(config, address.as_str());
        let child = command
            .spawn()
            .map_err(|error| format!("{RUNTIME_ERROR}: node spawn failed: {error}"))?;
        let mut runtime = Self { child };
        if let Err(error) = wait_until_ready(&mut runtime.child, address.as_str()) {
            runtime.cleanup();
            return Err(error);
        }
        Ok(runtime)
    }

    pub(super) fn cleanup(&mut self) {
        terminate_process(&mut self.child);
    }
}

impl Drop for LocalRuntime {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn runtime_command(config: &AgentTransactionDemoConfig, address: &str) -> Command {
    let storage = std::path::Path::new(config.staging_root.as_str()).join("node-state");
    let mut command = Command::new(config.local_node_binary.as_str());
    command.args(runtime_args(address));
    command
        .arg(storage)
        .env("KAMN_SERVICE_API_TLS_MODE", "disabled");
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    command.process_group(0);
    command
}

fn runtime_args(address: &str) -> [&str; 11] {
    [
        "--runtime-mode",
        "api",
        "--role",
        "processor",
        "--api-bind",
        address,
        "--api-max-requests",
        "1000",
        "--api-idle-timeout-ms",
        "600000",
        "--storage-dir",
    ]
}

fn endpoint_address(endpoint: &str) -> Result<String, String> {
    let address = endpoint
        .strip_prefix("http://")
        .filter(|value| value.starts_with("127.0.0.1:") || value.starts_with("localhost:"));
    match address {
        Some(value) if value.rsplit_once(':').is_some() => Ok(value.to_owned()),
        _ => Err(format!("{RUNTIME_ERROR}: endpoint must be loopback HTTP")),
    }
}

fn wait_until_ready(child: &mut Child, address: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if TcpStream::connect(address).is_ok() {
            return Ok(());
        }
        if let Some(status) = child.try_wait().map_err(runtime_wait_error)? {
            return Err(format!("{RUNTIME_ERROR}: node exited with {status}"));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(format!("{RUNTIME_ERROR}: node readiness timed out"))
}

fn runtime_wait_error(error: std::io::Error) -> String {
    format!("{RUNTIME_ERROR}: node status failed: {error}")
}
