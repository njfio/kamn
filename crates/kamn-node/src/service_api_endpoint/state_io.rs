use super::ServiceApiRelaySpoolEntry;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};

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
    fs::write(path, "")
        .map_err(|error| format!("service api relay spool truncate failed: {path}: {error}"))?;
    Ok(entries)
}

pub(crate) fn project_service_api_relayed_message_statuses(
    state_file: Option<&str>,
    message_ids: &[String],
) -> Result<usize, String> {
    let Some(path) = state_file else {
        return Ok(0);
    };
    if message_ids.is_empty() {
        return Ok(0);
    }
    let state_payload = match fs::read_to_string(path) {
        Ok(payload) => payload,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => {
            return Err(format!(
                "service api state file read failed: {path}: {error}"
            ));
        }
    };
    let mut state_json: serde_json::Value = serde_json::from_str(state_payload.as_str())
        .map_err(|error| format!("service api state file parse failed: {path}: {error}"))?;
    let Some(messages) = state_json
        .get_mut("messages")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return Err(format!(
            "service api state file parse failed: {path}: missing messages object"
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
        fs::write(path, rendered)
            .map_err(|error| format!("service api state file write failed: {path}: {error}"))?;
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
