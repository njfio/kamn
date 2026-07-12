use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::{build_pi_actor_command, AgentTransactionDemoConfig, AgentTransactionRole};

const CHILD_ERROR: &str = "AGENT_TRANSACTION_CHILD_FAILED";

struct RpcChild {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

pub(super) fn run_supervised_registration(
    config: &AgentTransactionDemoConfig,
) -> Result<String, String> {
    let mut children = spawn_children(config)?;
    let result = prompt(
        &mut children[1],
        "Register Agent B using the only registration tool.",
    );
    cleanup_all(&mut children);
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
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| child_error("failed to spawn Pi actor"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| child_error("missing Pi stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| child_error("missing Pi stdout"))?;
    Ok(RpcChild {
        child,
        stdin: Some(stdin),
        stdout: BufReader::new(stdout),
    })
}

fn prompt(child: &mut RpcChild, message: &str) -> Result<(), String> {
    let request = serde_json::json!({"id":"phase","type":"prompt","message":message});
    let stdin = child
        .stdin
        .as_mut()
        .ok_or_else(|| child_error("Pi stdin closed"))?;
    writeln!(stdin, "{request}").map_err(|_| child_error("Pi prompt write failed"))?;
    stdin
        .flush()
        .map_err(|_| child_error("Pi prompt flush failed"))?;
    read_agent_end(child)
}

fn read_agent_end(child: &mut RpcChild) -> Result<(), String> {
    let mut line = String::new();
    loop {
        line.clear();
        if child
            .stdout
            .read_line(&mut line)
            .map_err(|_| child_error("Pi RPC read failed"))?
            == 0
        {
            return Err(child_error("Pi actor exited before agent_end"));
        }
        let event: Value = serde_json::from_str(line.trim())
            .map_err(|_| child_error("Pi RPC emitted malformed JSON"))?;
        if event["type"] == "extension_error" {
            return Err(child_error("Pi extension failed"));
        }
        if event["type"] == "agent_end" {
            return Ok(());
        }
    }
}

fn cleanup_all(children: &mut [RpcChild; 3]) {
    for child in children.iter_mut() {
        child.stdin.take();
    }
    for child in children.iter_mut() {
        wait_or_kill(&mut child.child);
    }
}

fn wait_or_kill(child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = child.kill();
    let _ = child.wait();
}

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
