use super::super::super::*;
use kamn_core::SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV;
use std::collections::BTreeMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV: &str =
    "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON";
#[cfg(test)]
pub(super) const SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST: &str =
    SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV;

pub(super) fn resolve_daemon_service_api_relay_recipient_route_map(
) -> Result<BTreeMap<String, String>, ConfigError> {
    let raw = optional_route_map_env()?;
    let Some(raw) = raw else {
        return Ok(BTreeMap::new());
    };
    let parsed = parse_route_map_json(raw.as_str())?;
    normalize_route_map_entries(parsed)
}

pub(super) fn resolve_daemon_service_api_auth_private_key_hex() -> Result<String, ConfigError> {
    match env::var(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV) {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                    "{SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV} must not be empty when relay forwarding is enabled"
                )));
            }
            Ok(normalized.to_owned())
        }
        Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV} is required when {SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} is configured"
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV} must be valid utf-8 when present"
        ))),
    }
}

fn optional_route_map_env() -> Result<Option<String>, ConfigError> {
    match env::var(SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV) {
        Ok(value) => Ok(Some(non_empty_route_map_env(value)?)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} must be valid utf-8 when present"
        ))),
    }
}

fn non_empty_route_map_env(raw: String) -> Result<String, ConfigError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} must not be empty when present"
        )));
    }
    Ok(normalized.to_owned())
}

fn parse_route_map_json(raw: &str) -> Result<BTreeMap<String, String>, ConfigError> {
    serde_json::from_str::<BTreeMap<String, String>>(raw).map_err(|error| {
        ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} must be a JSON object mapping recipient DID to relay address: {error}"
        ))
    })
}

fn normalize_route_map_entries(
    parsed: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, ConfigError> {
    let mut routes = BTreeMap::new();
    for (recipient_did, relay_addr) in parsed {
        let (recipient_did, relay_addr) =
            normalize_route_entry(recipient_did.as_str(), relay_addr.as_str())?;
        routes.insert(recipient_did, relay_addr);
    }
    Ok(routes)
}

fn normalize_route_entry(
    recipient_did: &str,
    relay_addr: &str,
) -> Result<(String, String), ConfigError> {
    let normalized_recipient_did = recipient_did.trim();
    if normalized_recipient_did.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} contains an empty recipient DID key"
        )));
    }
    let normalized_relay_addr = relay_addr.trim();
    if normalized_relay_addr.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} contains an empty relay address value for recipient={normalized_recipient_did}"
        )));
    }
    Ok((
        normalized_recipient_did.to_owned(),
        normalized_relay_addr.to_owned(),
    ))
}

pub(super) fn initial_daemon_relay_nonce_counter() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(1)
}
