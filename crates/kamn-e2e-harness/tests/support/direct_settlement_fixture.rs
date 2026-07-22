#![allow(clippy::duplicate_mod)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[path = "service_authority_fixture.rs"]
mod service_authority_fixture;

const PAYER: &str = "2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T";
const RECIPIENT: &str = "FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe";
const SIGNATURE: &str = service_authority_fixture::SIGNATURE;

pub(crate) struct PathGuard(String);

impl Drop for PathGuard {
    fn drop(&mut self) {
        std::env::set_var("PATH", self.0.as_str());
    }
}

pub(crate) fn install(root: &Path) -> PathGuard {
    write_state(root);
    write_executable(
        root.join("solana-keygen").as_path(),
        format!("#!/bin/sh\necho {PAYER}\n").as_str(),
    );
    write_executable(root.join("solana").as_path(), solana_script().as_str());
    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{original}", root.display()));
    PathGuard(original)
}

pub(crate) fn state_source(root: &Path) -> PathBuf {
    root.join("persisted-service-state.json")
}

fn write_state(root: &Path) {
    let state = service_authority_fixture::state(RECIPIENT);
    std::fs::write(
        state_source(root),
        serde_json::to_vec(&state).expect("state JSON"),
    )
    .expect("persisted state");
}

fn solana_script() -> String {
    format!(
        r#"#!/bin/sh
cat <<'JSON'
{{"confirmationStatus":"finalized","meta":{{"err":null,"fee":5000,"preBalances":[2500000000,2500000000],"postBalances":[2498995000,2501000000]}},"transaction":{{"signatures":["{SIGNATURE}"],"message":{{"accountKeys":["{PAYER}","{RECIPIENT}"]}}}}}}
JSON
"#
    )
}

fn write_executable(path: &Path, body: &str) {
    std::fs::write(path, body).expect("fake executable");
    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}
