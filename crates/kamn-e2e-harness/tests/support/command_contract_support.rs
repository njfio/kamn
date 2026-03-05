#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt};

pub fn temp_path(name: &str) -> PathBuf {
    static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let seq = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("kamn-e2e-harness-{pid}-{nanos}-{seq}-{name}"))
}

pub fn valid_chain_dump_json() -> &'static str {
    r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:block-0"}]}"#
}

pub fn write_stub_binary(path: &PathBuf) {
    std::fs::write(path, "#!/bin/sh\nexit 0\n").expect("stub binary should be created");
}

pub fn write_failing_stub_binary(path: &PathBuf) {
    std::fs::write(path, "#!/bin/sh\nexit 1\n").expect("failing stub binary should be created");
}

#[cfg(unix)]
pub fn set_executable(path: &PathBuf) {
    let mut permissions = fs::metadata(path)
        .expect("binary metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("binary should become executable");
}

#[cfg(unix)]
pub fn set_non_executable(path: &PathBuf) {
    let mut permissions = fs::metadata(path)
        .expect("binary metadata should exist")
        .permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(path, permissions).expect("binary should become non-executable");
}

fn external_component_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn with_external_component_binaries<R>(f: impl FnOnce() -> R) -> R {
    let _env_guard = external_component_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let processor_binary = temp_path("kamn-processor-binary");
    let listener_binary = temp_path("kamn-listener-binary");
    let approver_binary = temp_path("kamn-approver-binary");

    write_stub_binary(&processor_binary);
    write_stub_binary(&listener_binary);
    write_stub_binary(&approver_binary);
    #[cfg(unix)]
    {
        set_executable(&processor_binary);
        set_executable(&listener_binary);
        set_executable(&approver_binary);
    }

    let vars = [
        (
            "KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY",
            processor_binary.display().to_string(),
        ),
        (
            "KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY",
            listener_binary.display().to_string(),
        ),
        (
            "KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY",
            approver_binary.display().to_string(),
        ),
    ];
    let previous = vars
        .iter()
        .map(|(key, _)| ((*key).to_owned(), std::env::var(key).ok()))
        .collect::<Vec<_>>();
    for (key, value) in vars {
        std::env::set_var(key, value);
    }

    let result = f();

    for (key, previous_value) in previous {
        if let Some(value) = previous_value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }
    let _ = std::fs::remove_file(processor_binary);
    let _ = std::fs::remove_file(listener_binary);
    let _ = std::fs::remove_file(approver_binary);
    result
}
