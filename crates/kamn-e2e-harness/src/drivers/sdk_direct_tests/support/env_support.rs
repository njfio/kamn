use std::env;
use std::ffi::OsString;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
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

pub(crate) fn assert_probe_error_contains<F>(
    updates: &[(&str, Option<&str>)],
    expected: &str,
    runner: F,
) where
    F: FnOnce() -> Result<(), String>,
{
    with_env_vars(updates, || {
        let error = runner().expect_err("probe should fail");
        assert!(
            error.contains(expected),
            "probe error should mention {expected}: {error}"
        );
    });
}

pub(crate) fn assert_probe_error_matches_any<F>(
    updates: &[(&str, Option<&str>)],
    expected_a: &str,
    expected_b: &str,
    runner: F,
) where
    F: FnOnce() -> Result<(), String>,
{
    with_env_vars(updates, || {
        let error = runner().expect_err("probe should fail");
        assert!(
            error.contains(expected_a) || error.contains(expected_b),
            "probe error should mention {expected_a} or {expected_b}: {error}"
        );
    });
}

fn capture_previous_env(updates: &[(&str, Option<&str>)]) -> Vec<(String, Option<OsString>)> {
    updates
        .iter()
        .map(|(key, _)| ((*key).to_owned(), env::var_os(key)))
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

fn set_env_value(key: &str, value: Option<&str>) {
    match value {
        Some(value) => unsafe { env::set_var(key, value) },
        None => clear_env_value(key),
    }
}

fn set_os_env_value(key: &str, value: &OsString) {
    unsafe { env::set_var(key, value) };
}

fn clear_env_value(key: &str) {
    unsafe { env::remove_var(key) };
}
