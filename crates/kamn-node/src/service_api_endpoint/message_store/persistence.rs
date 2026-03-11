use super::*;
#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::path::Path;

impl ServiceApiMessageStore {
    pub(crate) fn from_optional_state_file(state_file: Option<String>) -> Result<Self, String> {
        let path_label = state_file.as_deref().unwrap_or("<none>");
        let snapshot = match load_service_api_state_payload(state_file.as_deref())? {
            Some(contents) => {
                serde_json::from_str::<ServiceApiPersistedMessageStoreSnapshot>(contents.as_str())
                    .map_err(|error| {
                    format!("service api state file parse failed: {path_label}: {error}")
                })?
            }
            None => ServiceApiPersistedMessageStoreSnapshot::default(),
        };
        Ok(Self {
            state_file,
            snapshot,
        })
    }

    pub(crate) fn persist(&self) -> Result<(), String> {
        let payload = serde_json::to_string_pretty(&self.snapshot)
            .map_err(|error| format!("service api state serialization failed: {error}"))?;
        persist_service_api_state_payload(self.state_file.as_deref(), payload.as_str())
    }

    pub(crate) fn refresh_from_disk(&mut self) -> Result<(), String> {
        let path_label = self.state_file.as_deref().unwrap_or("<none>");
        let payload = match load_service_api_state_payload(self.state_file.as_deref())? {
            Some(contents) => contents,
            None => return Ok(()),
        };
        let snapshot =
            serde_json::from_str::<ServiceApiPersistedMessageStoreSnapshot>(payload.as_str())
                .map_err(|error| {
                    format!("service api state file parse failed: {path_label}: {error}")
                })?;
        self.snapshot = snapshot;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn write_state_file_atomically(path: &Path, payload: &str) -> Result<(), String> {
    let parent_dir = path
        .parent()
        .filter(|candidate| !candidate.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "service api state file path has no file name: {}",
                path.display()
            )
        })?;
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("service api state file temp suffix failed: {error}"))?
        .as_nanos();
    let temp_file_name = format!("{file_name}.tmp-{}-{unique_suffix}", std::process::id());
    let temp_path = parent_dir.join(temp_file_name);

    let mut temp_file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temp_path.as_path())
        .map_err(|error| {
            format!(
                "service api state file temp create failed: {}: {error}",
                temp_path.display()
            )
        })?;

    if let Err(error) = temp_file.write_all(payload.as_bytes()) {
        let _ = fs::remove_file(temp_path.as_path());
        return Err(format!(
            "service api state file temp write failed: {}: {error}",
            temp_path.display()
        ));
    }

    if let Err(error) = temp_file.sync_all() {
        let _ = fs::remove_file(temp_path.as_path());
        return Err(format!(
            "service api state file temp sync failed: {}: {error}",
            temp_path.display()
        ));
    }
    drop(temp_file);

    if let Err(error) = fs::rename(temp_path.as_path(), path) {
        let _ = fs::remove_file(temp_path.as_path());
        return Err(format!(
            "service api state file rename failed: {}: {error}",
            path.display()
        ));
    }

    if let Ok(parent_handle) = fs::File::open(parent_dir) {
        let _ = parent_handle.sync_all();
    }

    Ok(())
}
