use super::ServiceApiRelaySpoolEntry;
use kamn_core::SqliteStoreBackend;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const SERVICE_API_STATE_SQLITE_NAMESPACE: &str = "service_api_state";
pub(crate) const SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY: &str = "message_store_snapshot";

enum ServiceApiStateStorageBackend<'a> {
    JsonFile(&'a str),
    SqliteFile(&'a str),
}

fn service_api_state_storage_backend(
    state_file: Option<&str>,
) -> Option<ServiceApiStateStorageBackend<'_>> {
    let path = state_file?;
    if path.trim().is_empty() {
        return None;
    }
    if service_api_state_file_is_sqlite(path) {
        return Some(ServiceApiStateStorageBackend::SqliteFile(path));
    }
    Some(ServiceApiStateStorageBackend::JsonFile(path))
}

fn service_api_state_file_is_sqlite(path: &str) -> bool {
    let normalized = path.trim().to_ascii_lowercase();
    normalized.ends_with(".sqlite")
        || normalized.ends_with(".sqlite3")
        || normalized.ends_with(".db")
}

fn atomic_temp_filename_prefix(target: &Path) -> String {
    let file_name = target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "state".to_owned());
    format!(".{file_name}.tmp-{}-", std::process::id())
}

fn atomic_temp_path_for_target(target: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let temp_name = format!("{}{}", atomic_temp_filename_prefix(target), nonce);
    target.with_file_name(temp_name)
}

fn cleanup_atomic_temp_file(temp_path: &Path) {
    let _ = fs::remove_file(temp_path);
}

fn sync_parent_directory_best_effort(target: &Path) {
    let Some(parent) = target.parent() else {
        return;
    };
    if parent.as_os_str().is_empty() {
        return;
    }
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
}

fn write_file_atomically(path: &str, payload: &[u8], error_prefix: &str) -> Result<(), String> {
    let target = Path::new(path);
    let temp_path = atomic_temp_path_for_target(target);
    let mut temp_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path.as_path())
        .map_err(|error| format!("{error_prefix}: {path}: {error}"))?;
    if let Err(error) = temp_file.write_all(payload) {
        cleanup_atomic_temp_file(temp_path.as_path());
        return Err(format!("{error_prefix}: {path}: {error}"));
    }
    if let Err(error) = temp_file.sync_all() {
        cleanup_atomic_temp_file(temp_path.as_path());
        return Err(format!("{error_prefix}: {path}: {error}"));
    }
    drop(temp_file);
    if let Err(error) = fs::rename(temp_path.as_path(), target) {
        cleanup_atomic_temp_file(temp_path.as_path());
        return Err(format!("{error_prefix}: {path}: {error}"));
    }
    sync_parent_directory_best_effort(target);
    Ok(())
}

pub(super) fn load_service_api_state_payload(
    state_file: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(backend) = service_api_state_storage_backend(state_file) else {
        return Ok(None);
    };
    match backend {
        ServiceApiStateStorageBackend::JsonFile(path) => match fs::read_to_string(path) {
            Ok(payload) => Ok(Some(payload)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!(
                "service api state file read failed: {path}: {error}"
            )),
        },
        ServiceApiStateStorageBackend::SqliteFile(path) => {
            let backend = SqliteStoreBackend::open(Path::new(path)).map_err(|error| {
                format!("service api sqlite state open failed: {path}: {error}")
            })?;
            let maybe_payload = backend
                .get(
                    SERVICE_API_STATE_SQLITE_NAMESPACE,
                    SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY,
                )
                .map_err(|error| {
                    format!("service api sqlite state read failed: {path}: {error}")
                })?;
            if let Some(payload) = maybe_payload {
                let rendered = String::from_utf8(payload).map_err(|error| {
                    format!("service api sqlite state payload utf-8 decode failed: {path}: {error}")
                })?;
                return Ok(Some(rendered));
            }
            Ok(None)
        }
    }
}

pub(super) fn persist_service_api_state_payload(
    state_file: Option<&str>,
    payload: &str,
) -> Result<(), String> {
    let Some(backend) = service_api_state_storage_backend(state_file) else {
        return Ok(());
    };
    match backend {
        ServiceApiStateStorageBackend::JsonFile(path) => write_file_atomically(
            path,
            payload.as_bytes(),
            "service api state file write failed",
        ),
        ServiceApiStateStorageBackend::SqliteFile(path) => {
            let mut backend = SqliteStoreBackend::open(Path::new(path)).map_err(|error| {
                format!("service api sqlite state open failed: {path}: {error}")
            })?;
            backend
                .put(
                    SERVICE_API_STATE_SQLITE_NAMESPACE,
                    SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY,
                    payload.as_bytes(),
                )
                .map_err(|error| format!("service api sqlite state write failed: {path}: {error}"))
        }
    }
}

pub(crate) fn default_service_api_state_file_path_for_bind_addr(bind_addr: &str) -> String {
    let mut path = env::temp_dir();
    let bind_label = sanitize_service_api_state_file_component(bind_addr);
    path.push(format!("kamn-node-service-api-state-{bind_label}.json"));
    path.to_string_lossy().to_string()
}

pub(crate) fn default_service_api_relay_spool_file_path_from_state_file(
    state_file: &str,
) -> String {
    format!("{state_file}.relay.ndjson")
}

pub(crate) fn default_service_api_replay_guard_state_file_path_from_state_file(
    state_file: &str,
) -> String {
    format!("{state_file}.replay-guard.json")
}

pub(crate) fn append_service_api_relay_spool_entry(
    relay_spool_file: Option<&str>,
    entry: &ServiceApiRelaySpoolEntry,
) -> Result<(), String> {
    let Some(path) = relay_spool_file else {
        return Ok(());
    };
    let payload = serde_json::to_string(entry)
        .map_err(|error| format!("service api relay spool serialization failed: {error}"))?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("service api relay spool open failed: {path}: {error}"))?;
    writeln!(file, "{payload}")
        .map_err(|error| format!("service api relay spool append failed: {path}: {error}"))?;
    Ok(())
}

pub(crate) fn drain_service_api_relay_spool_entries(
    relay_spool_file: Option<&str>,
) -> Result<Vec<ServiceApiRelaySpoolEntry>, String> {
    let Some(path) = relay_spool_file else {
        return Ok(Vec::new());
    };
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "service api relay spool read failed: {path}: {error}"
            ));
        }
    };
    let mut entries = Vec::new();
    let reader = BufReader::new(file);
    for (line_index, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|error| format!("service api relay spool read failed: {path}: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let entry =
            serde_json::from_str::<ServiceApiRelaySpoolEntry>(line.as_str()).map_err(|error| {
                format!(
                    "service api relay spool parse failed: {path}: line {}: {error}",
                    line_index + 1
                )
            })?;
        entries.push(entry);
    }
    write_file_atomically(path, b"", "service api relay spool truncate failed")?;
    Ok(entries)
}

pub(crate) fn project_service_api_relayed_message_statuses(
    state_file: Option<&str>,
    message_ids: &[String],
) -> Result<usize, String> {
    if message_ids.is_empty() {
        return Ok(0);
    }
    let state_payload = match load_service_api_state_payload(state_file)? {
        Some(payload) => payload,
        None => return Ok(0),
    };
    let path_label = state_file.unwrap_or("<none>");
    let mut state_json: serde_json::Value = serde_json::from_str(state_payload.as_str())
        .map_err(|error| format!("service api state file parse failed: {path_label}: {error}"))?;
    let Some(messages) = state_json
        .get_mut("messages")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return Err(format!(
            "service api state file parse failed: {path_label}: missing messages object"
        ));
    };

    let unique_ids: BTreeSet<&str> = message_ids.iter().map(String::as_str).collect();
    let mut projected_count = 0_usize;
    for message_id in unique_ids {
        let Some(record) = messages
            .get_mut(message_id)
            .and_then(serde_json::Value::as_object_mut)
        else {
            continue;
        };
        let status = record.get("status").and_then(serde_json::Value::as_str);
        if status == Some("created") {
            record.insert(
                "status".to_owned(),
                serde_json::Value::String("relayed".to_owned()),
            );
            projected_count = projected_count.saturating_add(1);
        }
    }

    if projected_count > 0 {
        let rendered = serde_json::to_string_pretty(&state_json)
            .map_err(|error| format!("service api state serialization failed: {error}"))?;
        persist_service_api_state_payload(state_file, rendered.as_str())?;
    }
    Ok(projected_count)
}

fn sanitize_service_api_state_file_component(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut last_was_separator = false;
    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if normalized == '-' {
            if !last_was_separator {
                output.push(normalized);
            }
            last_was_separator = true;
            continue;
        }
        output.push(normalized);
        last_was_separator = false;
    }
    let trimmed = output.trim_matches('-');
    if trimmed.is_empty() {
        "default".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        atomic_temp_filename_prefix, drain_service_api_relay_spool_entries,
        persist_service_api_state_payload, write_file_atomically,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file_path(label: &str, suffix: &str) -> PathBuf {
        let unique_suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be monotonic")
                .as_nanos()
        );
        std::env::temp_dir().join(format!(
            "kamn-node-state-io-{label}-{unique_suffix}.{suffix}"
        ))
    }

    fn temp_artifacts_for_target(target: &Path) -> Vec<PathBuf> {
        let Some(parent) = target.parent() else {
            return Vec::new();
        };
        let prefix = atomic_temp_filename_prefix(target);
        let mut artifacts = Vec::new();
        let Ok(entries) = fs::read_dir(parent) else {
            return artifacts;
        };
        for entry_result in entries {
            let Ok(entry) = entry_result else {
                continue;
            };
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.starts_with(prefix.as_str()) {
                artifacts.push(entry.path());
            }
        }
        artifacts
    }

    #[test]
    fn unit_write_file_atomically_replaces_content_without_leaking_temp_files() {
        let state_file = unique_temp_file_path("atomic-replace", "json");
        fs::write(state_file.as_path(), r#"{"state":"old"}"#).expect("fixture should write");
        write_file_atomically(
            state_file.to_str().expect("state path should be utf-8"),
            br#"{"state":"new"}"#,
            "atomic test write failed",
        )
        .expect("atomic write should replace destination");
        let payload =
            fs::read_to_string(state_file.as_path()).expect("replaced payload should be readable");
        assert_eq!(payload, r#"{"state":"new"}"#);
        assert!(
            temp_artifacts_for_target(state_file.as_path()).is_empty(),
            "atomic write should not leave temp artifacts beside destination"
        );
        let _ = fs::remove_file(state_file);
    }

    #[test]
    fn regression_persist_service_api_state_payload_json_backend_is_atomic() {
        // Regression: #6110
        let state_file = unique_temp_file_path("persist-atomic", "json");
        fs::write(state_file.as_path(), r#"{"schema_version":"old"}"#)
            .expect("fixture should write");
        persist_service_api_state_payload(
            state_file.to_str(),
            r#"{"schema_version":"kamn.runtime.service-api-message-store.v2"}"#,
        )
        .expect("json backend persistence should succeed");
        let payload =
            fs::read_to_string(state_file.as_path()).expect("persisted payload should be readable");
        assert_eq!(
            payload,
            r#"{"schema_version":"kamn.runtime.service-api-message-store.v2"}"#
        );
        assert!(
            temp_artifacts_for_target(state_file.as_path()).is_empty(),
            "json persistence should not leave atomic temp artifacts"
        );
        let _ = fs::remove_file(state_file);
    }

    #[test]
    fn regression_relay_spool_drain_truncates_with_atomic_replace() {
        // Regression: #6110
        let spool_file = unique_temp_file_path("relay-spool", "ndjson");
        fs::write(
            spool_file.as_path(),
            r#"{"message_id":"msg-1","sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"hello\"}","queued_at_unix":1}
"#,
        )
        .expect("relay spool fixture should write");
        let drained = drain_service_api_relay_spool_entries(spool_file.to_str())
            .expect("relay spool drain should succeed");
        assert_eq!(drained.len(), 1);
        let payload = fs::read_to_string(spool_file.as_path())
            .expect("relay spool file should remain readable after drain");
        assert!(
            payload.is_empty(),
            "relay spool drain should atomically replace with empty payload"
        );
        assert!(
            temp_artifacts_for_target(spool_file.as_path()).is_empty(),
            "relay spool drain should not leak temp artifacts"
        );
        let _ = fs::remove_file(spool_file);
    }
}
