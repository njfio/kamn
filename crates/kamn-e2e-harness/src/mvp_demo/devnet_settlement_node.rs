use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use super::devnet_settlement_build::node_binary_path;
use super::devnet_settlement_live::LiveSettlementConfig;
use super::devnet_settlement_service::drive_escrow_release;

pub(super) struct ServiceApiRun {
    pub(super) escrow_id: String,
}

pub(super) fn launch_and_drive_service_api(
    run_dir: &Path,
    state_file: &Path,
    config: &LiveSettlementConfig,
) -> Result<ServiceApiRun, String> {
    let port = reserve_local_port()?;
    let endpoint = format!("http://127.0.0.1:{port}");
    let mut child = spawn_node(run_dir, state_file, port, config)?;
    wait_for_tcp_ready(port, &mut child)?;
    let escrow_id = match drive_escrow_release(endpoint.as_str(), run_dir) {
        Ok(value) => value,
        Err(error) => {
            terminate_child(&mut child);
            return Err(error);
        }
    };
    wait_for_child_success(&mut child)?;
    Ok(ServiceApiRun { escrow_id })
}

fn spawn_node(
    run_dir: &Path,
    state_file: &Path,
    port: u16,
    config: &LiveSettlementConfig,
) -> Result<Child, String> {
    let stdout = proof_file(run_dir, "devnet-settlement-node-stdout.txt")?;
    let stderr = proof_file(run_dir, "devnet-settlement-node-stderr.txt")?;
    let mut command = Command::new(node_binary_path());
    command.args(node_args(run_dir, port));
    apply_node_env(&mut command, run_dir, state_file, config);
    command
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    command
        .spawn()
        .map_err(|error| format!("failed to launch devnet settlement service API node: {error}"))
}

fn apply_node_env(
    command: &mut Command,
    run_dir: &Path,
    state_file: &Path,
    config: &LiveSettlementConfig,
) {
    command.env("KAMN_SERVICE_API_TLS_MODE", "disabled");
    command.env("KAMN_SERVICE_API_STATE_FILE", state_file);
    command.env(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        run_dir.join("state/relay-spool.json"),
    );
    command.env(
        "KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL",
        &config.rpc_url,
    );
    command.env(
        "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE",
        &config.keypair_file,
    );
    command.env(
        "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY",
        &config.recipient_pubkey,
    );
    command.env(
        "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS",
        config.lamports.to_string(),
    );
    command.env(
        "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT",
        &config.commitment,
    );
}

fn node_args(run_dir: &Path, port: u16) -> Vec<String> {
    vec![
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--api-bind".to_owned(),
        format!("127.0.0.1:{port}"),
        "--api-max-requests".to_owned(),
        "2".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "120000".to_owned(),
        "--storage-dir".to_owned(),
        run_dir.join("state/node-storage").display().to_string(),
        "--disable-gossip".to_owned(),
        "--output".to_owned(),
        "text".to_owned(),
        "--chain-id".to_owned(),
        "kamn-mvp-demo".to_owned(),
    ]
}

fn proof_file(run_dir: &Path, name: &str) -> Result<File, String> {
    File::create(run_dir.join("proof").join(name))
        .map_err(|error| format!("failed to create devnet settlement node log {name}: {error}"))
}

fn reserve_local_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("failed to reserve local service API port: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("failed to inspect reserved local port: {error}"))?
        .port();
    drop(listener);
    Ok(port)
}

fn wait_for_tcp_ready(port: u16, child: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        reject_early_exit(child)?;
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("service API node did not become ready in time".to_owned())
}

fn reject_early_exit(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("service API node exited before accepting connections".to_owned());
    }
    Ok(())
}

fn wait_for_child_success(child: &mut Child) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(150);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return status
                .success()
                .then_some(())
                .ok_or_else(|| format!("service API node exited with status {status}"));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = child.kill();
    Err("service API node did not exit after request budget".to_owned())
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}
