use super::super::super::*;
use std::path::{Path, PathBuf};

pub(crate) fn unique_named_state_file(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ))
}

pub(crate) fn read_state_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state payload should parse")
}

pub(crate) fn read_audit_export_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("audit export file should remain readable");
    serde_json::from_str(payload.as_str()).expect("audit export payload should parse")
}

pub(crate) fn default_audit_export_file(state_file: &Path) -> PathBuf {
    PathBuf::from(format!("{}.audit-export.json", state_file.to_string_lossy()))
}

pub(crate) fn set_state_file_env(path: &Path) -> (String, EnvVarGuard) {
    let path_text = path.to_string_lossy().to_string();
    let guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(path_text.as_str()));
    (path_text, guard)
}

pub(crate) fn set_audit_export_file_env(path: &Path) -> (String, EnvVarGuard) {
    let path_text = path.to_string_lossy().to_string();
    let guard = EnvVarGuard::set("KAMN_SERVICE_API_AUDIT_EXPORT_FILE", Some(path_text.as_str()));
    (path_text, guard)
}
