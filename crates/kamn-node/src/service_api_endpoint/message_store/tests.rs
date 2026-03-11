#[cfg(test)]
mod atomic_state_write_tests {
    use super::super::persistence::write_state_file_atomically;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be available")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-node-{name}-{}-{nanos}", std::process::id()))
    }

    fn collect_atomic_temp_entries(dir: &Path, state_file_name: &str) -> Vec<PathBuf> {
        let prefix = format!("{state_file_name}.tmp-");
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(prefix.as_str())
                {
                    entries.push(entry.path());
                }
            }
        }
        entries
    }

    #[test]
    fn unit_atomic_state_write_replaces_payload_and_removes_temp_entries() {
        let base_dir = unique_temp_dir("atomic-state-write-ok");
        fs::create_dir_all(base_dir.as_path()).expect("temp base dir should create");
        let state_file = base_dir.join("service-api-state.json");
        fs::write(state_file.as_path(), "{\"schema_version\":\"old\"}")
            .expect("initial state fixture should write");

        write_state_file_atomically(
            state_file.as_path(),
            "{\"schema_version\":\"new\",\"messages\":{}}",
        )
        .expect("atomic write should succeed");

        let payload = fs::read_to_string(state_file.as_path()).expect("state file should remain");
        assert!(
            payload.contains("\"schema_version\":\"new\""),
            "atomic replacement should update destination payload"
        );

        let temp_entries =
            collect_atomic_temp_entries(base_dir.as_path(), "service-api-state.json");
        assert!(
            temp_entries.is_empty(),
            "atomic writer should not leave temp files behind after success"
        );

        let _ = fs::remove_file(state_file);
        let _ = fs::remove_dir(base_dir);
    }

    #[test]
    fn unit_atomic_state_write_rename_failure_cleans_temp_entry() {
        let base_dir = unique_temp_dir("atomic-state-write-rename-fail");
        fs::create_dir_all(base_dir.as_path()).expect("temp base dir should create");
        let state_path = base_dir.join("service-api-state.json");
        fs::create_dir(state_path.as_path()).expect("fixture directory should create");

        let error = write_state_file_atomically(
            state_path.as_path(),
            "{\"schema_version\":\"new\",\"messages\":{}}",
        )
        .expect_err("rename over directory destination must fail");
        assert!(
            error.contains("state file rename failed"),
            "rename failure should fail closed with deterministic marker"
        );

        let temp_entries =
            collect_atomic_temp_entries(base_dir.as_path(), "service-api-state.json");
        assert!(
            temp_entries.is_empty(),
            "failed atomic write must clean temp entries before returning"
        );

        let _ = fs::remove_dir(state_path);
        let _ = fs::remove_dir(base_dir);
    }
}
