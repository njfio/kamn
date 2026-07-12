use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::Value;

use super::agent_transaction_evidence::AgentTransactionEvidencePaths;
use super::agent_transaction_process::{start_process, take_pipes, terminate_process_group};
use super::{build_pi_actor_command, AgentTransactionDemoConfig, AgentTransactionRole};

const CHILD_ERROR: &str = "AGENT_TRANSACTION_CHILD_FAILED";

struct RpcChild {
    child: Child,
    stdin: Option<ChildStdin>,
    events: Receiver<Result<Value, String>>,
    reader: Option<JoinHandle<()>>,
}

pub(super) struct RpcGroup {
    children: Vec<RpcChild>,
    timeout_ms: u64,
    cleaned: bool,
}

impl RpcGroup {
    pub(super) fn spawn(
        config: &AgentTransactionDemoConfig,
        paths: &AgentTransactionEvidencePaths,
    ) -> Result<Self, String> {
        let mut children = Vec::with_capacity(3);
        for role in [
            AgentTransactionRole::AgentA,
            AgentTransactionRole::AgentB,
            AgentTransactionRole::AgentC,
        ] {
            match spawn_child(config, paths, role) {
                Ok(child) => children.push(child),
                Err(error) => {
                    cleanup_children(children.as_mut_slice());
                    return Err(error);
                }
            }
        }
        Ok(Self {
            children,
            timeout_ms: config.rpc_timeout_ms,
            cleaned: false,
        })
    }

    pub(super) fn prompt(&mut self, index: usize, message: &str) -> Result<Value, String> {
        prompt(&mut self.children[index], message, self.timeout_ms)
    }

    pub(super) fn cleanup(&mut self) {
        if self.cleaned {
            return;
        }
        self.cleaned = true;
        cleanup_children(self.children.as_mut_slice());
    }
}

impl Drop for RpcGroup {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn spawn_child(
    config: &AgentTransactionDemoConfig,
    paths: &AgentTransactionEvidencePaths,
    role: AgentTransactionRole,
) -> Result<RpcChild, String> {
    let command = build_pi_actor_command(config, role);
    let environment = paths.environment(config);
    let mut child = start_process(command.as_slice(), environment.as_slice())?;
    let (stdin, stdout) = take_pipes(&mut child)?;
    let (events, reader) = spawn_reader(stdout);
    Ok(RpcChild {
        child,
        stdin: Some(stdin),
        events,
        reader: Some(reader),
    })
}

fn prompt(child: &mut RpcChild, message: &str, timeout_ms: u64) -> Result<Value, String> {
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

fn read_agent_end(child: &mut RpcChild, timeout_ms: u64) -> Result<Value, String> {
    let timeout = Duration::from_millis(timeout_ms);
    let mut tool_result = None;
    loop {
        let event = receive_event(child, timeout)?;
        if event["type"] == "extension_error" {
            return Err(child_error("Pi extension failed"));
        }
        if let Some(error) = failed_tool_error(&event) {
            return Err(child_error(error.as_str()));
        }
        if event["type"] == "tool_execution_end" {
            tool_result = Some(event.clone());
        }
        if event["type"] == "agent_end" {
            return Ok(tool_result.unwrap_or(event));
        }
    }
}

fn failed_tool_error(event: &Value) -> Option<String> {
    let failed = event["type"] == "tool_execution_end"
        && (event["isError"].as_bool() == Some(true)
            || event["result"]["isError"].as_bool() == Some(true));
    failed.then(|| {
        format!(
            "Pi tool failed: {}: {}",
            event["toolName"].as_str().unwrap_or("unknown"),
            event["result"]
        )
    })
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
    let handle = std::thread::spawn(move || read_events(stdout, sender));
    (receiver, handle)
}

fn read_events(stdout: ChildStdout, sender: mpsc::Sender<Result<Value, String>>) {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let event = serde_json::from_str(line.trim())
                    .map_err(|_| child_error("Pi RPC emitted malformed JSON"));
                if sender.send(event).is_err() {
                    break;
                }
            }
        }
    }
}

fn cleanup_children(children: &mut [RpcChild]) {
    for child in children.iter_mut() {
        child.stdin.take();
    }
    for child in children.iter_mut() {
        terminate_process_group(&mut child.child);
        if let Some(reader) = child.reader.take() {
            let _ = reader.join();
        }
    }
}

fn child_error(detail: &str) -> String {
    format!("{CHILD_ERROR}: {detail}")
}
