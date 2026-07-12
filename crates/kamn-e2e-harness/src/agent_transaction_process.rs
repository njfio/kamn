use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub(super) fn start_process(
    command: &[String],
    environment: &[(&'static str, String)],
) -> Result<Child, String> {
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .envs(environment.iter().cloned());
    #[cfg(unix)]
    process.process_group(0);
    process
        .spawn()
        .map_err(|_| child_error("failed to spawn Pi actor"))
}

pub(super) fn take_pipes(child: &mut Child) -> Result<(ChildStdin, ChildStdout), String> {
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

pub(super) fn wait_or_kill(child: &mut Child, timeout_ms: u64) {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(2_000));
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

pub(super) fn terminate_process_group(child: &mut Child) {
    if child.try_wait().ok().flatten().is_some() {
        return;
    }
    signal_process_group(child.id(), "-TERM");
    wait_after_signal(child);
}

pub(super) fn terminate_process(child: &mut Child) {
    if child.try_wait().ok().flatten().is_some() {
        return;
    }
    let pid = child.id().to_string();
    let _ = Command::new("kill").args(["-TERM", pid.as_str()]).status();
    wait_after_signal(child);
}

fn wait_after_signal(child: &mut Child) {
    let deadline = Instant::now() + Duration::from_millis(50);
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    wait_or_kill(child, 0);
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    signal_process_group(pid, "-KILL");
}

#[cfg(unix)]
fn signal_process_group(pid: u32, signal: &str) {
    let group = format!("-{pid}");
    let _ = Command::new("kill").args([signal, group.as_str()]).status();
}

#[cfg(not(unix))]
fn signal_process_group(_pid: u32, _signal: &str) {}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}

fn child_error(detail: &str) -> String {
    format!("AGENT_TRANSACTION_CHILD_FAILED: {detail}")
}
