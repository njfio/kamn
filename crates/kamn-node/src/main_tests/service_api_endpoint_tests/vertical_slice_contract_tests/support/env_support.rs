use super::super::super::*;
use std::path::{Path, PathBuf};

pub(crate) struct VerticalSliceFiles {
    pub(crate) sender_state_file: PathBuf,
    pub(crate) sender_spool_file: PathBuf,
    pub(crate) recipient_state_file: PathBuf,
    pub(crate) recipient_spool_file: PathBuf,
}

impl VerticalSliceFiles {
    pub(crate) fn new() -> Self {
        Self {
            sender_state_file: unique_named_state_file("kamn-node-vertical-slice-sender-state"),
            sender_spool_file: unique_named_relay_spool_file("kamn-node-vertical-slice-sender-spool"),
            recipient_state_file: unique_named_state_file("kamn-node-vertical-slice-recipient-state"),
            recipient_spool_file: unique_named_relay_spool_file("kamn-node-vertical-slice-recipient-spool"),
        }
    }

    pub(crate) fn cleanup(self) {
        remove_file(self.sender_state_file);
        remove_file(self.sender_spool_file);
        remove_file(self.recipient_state_file);
        remove_file(self.recipient_spool_file);
    }
}

pub(crate) fn unique_named_state_file(prefix: &str) -> PathBuf {
    unique_named_path(prefix, "json")
}

pub(crate) fn unique_named_relay_spool_file(prefix: &str) -> PathBuf {
    unique_named_path(prefix, "ndjson")
}

pub(crate) fn read_state_json(path: &Path) -> Value {
    read_json(path, "state")
}

pub(crate) fn read_audit_export_json(path: &Path) -> Value {
    read_json(path, "audit export")
}

pub(crate) fn default_audit_export_file(state_file: &Path) -> PathBuf {
    PathBuf::from(format!("{}.audit-export.json", state_file.to_string_lossy()))
}

pub(crate) fn set_state_file_env(path: &Path) -> (String, EnvVarGuard) {
    let path_text = path.to_string_lossy().to_string();
    let guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(path_text.as_str()));
    (path_text, guard)
}

pub(crate) fn set_relay_spool_env(path: &Path) -> (String, EnvVarGuard) {
    let path_text = path.to_string_lossy().to_string();
    let guard = EnvVarGuard::set("KAMN_SERVICE_API_RELAY_SPOOL_FILE", Some(path_text.as_str()));
    (path_text, guard)
}

pub(crate) fn set_audit_export_file_env(path: &Path) -> (String, EnvVarGuard) {
    let path_text = path.to_string_lossy().to_string();
    let guard = EnvVarGuard::set("KAMN_SERVICE_API_AUDIT_EXPORT_FILE", Some(path_text.as_str()));
    (path_text, guard)
}

fn unique_named_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.{}",
        std::process::id(),
        timestamp_nanos(),
        extension,
    ))
}

fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos()
}

fn read_json(path: &Path, label: &str) -> Value {
    let payload = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("{label} file should remain readable: {error}")
    });
    serde_json::from_str(payload.as_str())
        .unwrap_or_else(|error| panic!("{label} payload should parse: {error}"))
}

fn remove_file(path: PathBuf) {
    let _ = fs::remove_file(path);
}
