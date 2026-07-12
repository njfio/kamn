use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use serde_json::Value;

use super::{build_pi_actor_command, AgentTransactionDemoConfig, AgentTransactionRole};

const CHILD_ERROR: &str = "AGENT_TRANSACTION_CHILD_FAILED";

struct RpcChild {
    child: Child,
    stdin: Option<ChildStdin>,
    events: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}

pub(super) fn run_supervised_registration(
    config: &AgentTransactionDemoConfig,
) -> Result<String, String> {
    let mut children = spawn_children(config)?;
    let result = prompt(
        &mut children[1],
        "Register Agent B using the only registration tool.",
        config.rpc_timeout_ms,
    );
    cleanup_all(&mut children, config.rpc_timeout_ms);
    match result {
        Ok(()) => fail_no_go(config, "scenario orchestration incomplete"),
        Err(error) => fail_no_go(config, error.as_str()),
    }
}

fn spawn_children(config: &AgentTransactionDemoConfig) -> Result<[RpcChild; 3], String> {
    Ok([
        spawn_child(config, AgentTransactionRole::AgentA)?,
        spawn_child(config, AgentTransactionRole::AgentB)?,
        spawn_child(config, AgentTransactionRole::AgentC)?,
    ])
}

fn spawn_child(
    config: &AgentTransactionDemoConfig,
    role: AgentTransactionRole,
) -> Result<RpcChild, String> {
    let command = build_pi_actor_command(config, role);
    let mut child = start_process(command.as_slice())?;
    let (stdin, stdout) = take_pipes(&mut child)?;
    let (events, reader) = spawn_reader(stdout);
    Ok(RpcChild {
        child,
        stdin: Some(stdin),
        events,
        reader: Some(reader),
    })
}

fn start_process(command: &[String]) -> Result<Child, String> {
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(unix)]
    process.process_group(0);
    process
        .spawn()
        .map_err(|_| child_error("failed to spawn Pi actor"))
}

fn take_pipes(child: &mut Child) -> Result<(ChildStdin, ChildStdout), String> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| child_error("missing Pi stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| child_error("missing Pi stdout"))?;
    Ok((stdin, stdout))
}

fn prompt(child: &mut RpcChild, message: &str, timeout_ms: u64) -> Result<(), String> {
    let request = serde_json::json!({"id":"phase","type":"prompt","message":message});
    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| child_error("Pi stdin closed"))?;
    writeln!(stdin, "{request}").map_err(|_| child_error("Pi prompt write failed"))?;
    stdin
        .flush()
        .map_err(|_| child_error("Pi prompt flush failed"))?;
    read_agent_end(child, timeout_ms)
}

fn read_agent_end(child: &mut RpcChild, timeout_ms: u64) -> Result<(), String> {
    let timeout = Duration::from_millis(timeout_ms);
    loop {
        let event = receive_event(child, timeout)?;
        if event["type"] == "extension_error" {
            return Err(child_error("Pi extension failed"));
        }
        if event["type"] == "agent_end" {
            return Ok(());
        }
    }
}

fn receive_event(child: &RpcChild, timeout: Duration) -> Result<Value, String> {
    match child.events.recv_timeout(timeout) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(child_error("Pi RPC phase timed out")),
        Err(RecvTimeoutError::Disconnected) => Err(child_error("Pi actor exited before agent_end")),
    }
}

fn spawn_reader(stdout: ChildStdout) -> (Receiver<Result<Value, String>>, JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let event = serde_json::from_str(line.trim())
                        .map_err(|_| child_error("Pi RPC emitted malformed JSON"));
                    if sender.send(event).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    (receiver, handle)
}

fn cleanup_all(children: &mut [RpcChild; 3], timeout_ms: u64) {
    for child in children.iter_mut() {
        child.stdin.take();
    }
    for child in children.iter_mut() {
        wait_or_kill(&mut child.child, timeout_ms);
        if let Some(reader) = child.reader.take() {
            let _ = reader.join();
        }
    }
}

fn wait_or_kill(child: &mut Child, timeout_ms: u64) {
    let grace = Duration::from_millis(timeout_ms.min(2_000));
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    kill_process_group(child.id());
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    let group = format!("-{pid}");
    let _ = Command::new("kill")
        .args(["-KILL", group.as_str()])
        .status();
}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}

fn fail_no_go(config: &AgentTransactionDemoConfig, detail: &str) -> Result<String, String> {
    let error = child_error(detail);
    let latest = Path::new(config.output_root.as_str()).join("latest");
    std::fs::create_dir_all(&latest)
        .map_err(|_| child_error("failed to create NO-GO output directory"))?;
    std::fs::write(
        latest.join("NO-GO.txt"),
        format!("decision=NO-GO\nreason={error}\n"),
    )
    .map_err(|_| child_error("failed to write NO-GO report"))?;
    Err(error)
}

fn child_error(detail: &str) -> String {
    format!("{CHILD_ERROR}: {detail}")
}
