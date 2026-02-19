use super::*;
#[cfg(unix)]
use crate::{configure_os_signal_test_triggers, OsSignalTestKind, OsSignalTestTrigger};

fn write_temp_node_config(contents: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let unique_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    path.push(format!(
        "kamn-node-config-layering-{}-{unique_suffix}.conf",
        std::process::id()
    ));
    std::fs::write(&path, contents).expect("temp config file should write");
    path
}

// runtime_tests structural budget shell only; keep domain tests in src/main_tests/runtime_tests/*.rs
include!("runtime_tests/arg_and_signer_policy_tests.rs");
include!("runtime_tests/logging_and_bootstrap_tests.rs");
include!("runtime_tests/runtime_mode_and_transport_profile_tests.rs");
include!("runtime_tests/full_supervisor_and_shutdown_tests.rs");
include!("runtime_tests/profile_and_config_layering_tests.rs");
include!("runtime_tests/kolme_live_execution_tests.rs");
