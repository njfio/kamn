use super::script_core_support::{remove_script, script_path_str, unique_temp_script_path};
use crate::drivers::mcp_agent::MCP_AGENT_BINARY_ENV;
use std::ffi::OsString;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::PoisonError;

pub(crate) fn with_env_vars<F>(updates: &[(&str, Option<&str>)], test: F)
where
    F: FnOnce(),
{
    let _guard = crate::drivers::test_env_lock()
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let previous = capture_previous_env(updates);
    apply_env_updates(updates);
    let result = catch_unwind(AssertUnwindSafe(test));
    restore_previous_env(&previous);
    if let Err(payload) = result {
        resume_unwind(payload);
    }
}

pub(crate) fn assert_missing_binary_probe_failure<F>(
    extra_updates: &[(&str, Option<&str>)],
    runner: F,
)
where
    F: FnOnce() -> Result<(), String>,
{
    let mut updates = vec![(MCP_AGENT_BINARY_ENV, Some("/definitely/missing/kamn-mcp-server"))];
    updates.extend_from_slice(extra_updates);
    with_env_vars(&updates, || assert_spawn_failure(runner()));
}

pub(crate) fn assert_scripted_probe_succeeds<FWrite, FRun>(
    stem: &str,
    write_script: FWrite,
    extra_updates: &[(&str, Option<&str>)],
    runner: FRun,
) where
    FWrite: FnOnce(&Path),
    FRun: FnOnce() -> Result<(), String>,
{
    with_scripted_probe_env(stem, write_script, extra_updates, || {
        runner().expect("scripted probe should succeed");
    });
}

pub(crate) fn assert_scripted_probe_error_contains<FWrite, FRun>(
    stem: &str,
    write_script: FWrite,
    extra_updates: &[(&str, Option<&str>)],
    expected: &str,
    runner: FRun,
) where
    FWrite: FnOnce(&Path),
    FRun: FnOnce() -> Result<(), String>,
{
    with_scripted_probe_env(stem, write_script, extra_updates, || {
        let error = runner().expect_err("scripted probe should fail");
        assert!(
            error.contains(expected),
            "error should mention {expected}: {error}"
        );
    });
}

fn with_scripted_probe_env<FWrite, FRun>(
    stem: &str,
    write_script: FWrite,
    extra_updates: &[(&str, Option<&str>)],
    run: FRun,
) where
    FWrite: FnOnce(&Path),
    FRun: FnOnce(),
{
    let script_path = unique_temp_script_path(stem);
    write_script(&script_path);
    let script_binary = script_path_str(&script_path).to_owned();
    let mut updates = vec![(MCP_AGENT_BINARY_ENV, Some(script_binary.as_str()))];
    updates.extend_from_slice(extra_updates);
    with_env_vars(&updates, run);
    remove_script(&script_path);
}

fn capture_previous_env(updates: &[(&str, Option<&str>)]) -> Vec<(String, Option<OsString>)> {
    updates
        .iter()
        .map(|(key, _)| ((*key).to_owned(), std::env::var_os(key)))
        .collect()
}

fn apply_env_updates(updates: &[(&str, Option<&str>)]) {
    for (key, value) in updates {
        set_env_value(key, *value);
    }
}

fn restore_previous_env(previous: &[(String, Option<OsString>)]) {
    for (key, value) in previous {
        match value.as_ref() {
            Some(value) => set_os_env_value(key, value),
            None => clear_env_value(key),
        }
    }
}

fn assert_spawn_failure(result: Result<(), String>) {
    let error = result.expect_err("missing binary should fail");
    assert!(
        error.contains("failed to spawn"),
        "error should reflect spawn failure: {error}"
    );
}

fn set_env_value(key: &str, value: Option<&str>) {
    match value {
        Some(value) => {
            unsafe { std::env::set_var(key, value) };
        }
        None => clear_env_value(key),
    }
}

fn set_os_env_value(key: &str, value: &OsString) {
    unsafe { std::env::set_var(key, value) };
}

fn clear_env_value(key: &str) {
    unsafe { std::env::remove_var(key) };
}
