use super::super::{
    build_kolme_live_managed_signing_key, encode_kolme_hex_lower,
    reset_cached_log_config_for_tests, KAMN_NODE_LOG_FORMAT_ENV, KAMN_NODE_LOG_LEVEL_ENV,
};
use serde_json::Value;
use std::env;
use std::sync::Mutex;

pub(crate) fn signer_env_lock() -> &'static Mutex<()> {
    crate::signer_test_env_lock()
}

pub(crate) fn lock_signer_env_guard() -> std::sync::MutexGuard<'static, ()> {
    signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

pub(crate) fn log_env_lock() -> &'static Mutex<()> {
    signer_env_lock()
}

fn maybe_reset_log_config_cache_for_env_key(key: &str) {
    if key == KAMN_NODE_LOG_LEVEL_ENV || key == KAMN_NODE_LOG_FORMAT_ENV {
        reset_cached_log_config_for_tests();
    }
}

pub(crate) fn managed_signer_public_key_hex(key_reference: &str) -> String {
    let signing_key = build_kolme_live_managed_signing_key(key_reference)
        .expect("managed signing key should derive");
    encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    )
}

pub(crate) struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    pub(crate) fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        maybe_reset_log_config_cache_for_env_key(key);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            env::set_var(self.key, previous);
        } else {
            env::remove_var(self.key);
        }
        maybe_reset_log_config_cache_for_env_key(self.key);
    }
}

fn project_json_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn find_json_field_value(body: &Value, field: &str) -> Option<String> {
    match body {
        Value::Object(map) => map
            .get(field)
            .and_then(project_json_value_to_string)
            .or_else(|| {
                map.values()
                    .find_map(|value| find_json_field_value(value, field))
            }),
        Value::Array(entries) => entries
            .iter()
            .find_map(|value| find_json_field_value(value, field)),
        _ => None,
    }
}

pub(crate) fn extract_json_string_field(body: &str, field: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(body).ok()?;
    find_json_field_value(&parsed, field)
}
