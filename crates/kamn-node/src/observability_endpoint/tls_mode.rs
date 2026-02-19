use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::BufReader;

const OBSERVABILITY_ENDPOINT_TLS_MODE_ENV: &str = "KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE";
const OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV: &str = "KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE";
const OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV: &str = "KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE";
const OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED: &str = "disabled";
const OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE: &str = "require";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ObservabilityEndpointTlsMode {
    Disabled,
    Require { cert_file: String, key_file: String },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ObservabilityEndpointTlsModeOverride {
    InvalidMode { mode: String },
    Require { cert_file: String, key_file: String },
}

thread_local! {
    static OBSERVABILITY_ENDPOINT_TLS_MODE_OVERRIDE_FOR_TESTS: RefCell<Option<ObservabilityEndpointTlsModeOverride>> =
        const { RefCell::new(None) };
}

#[allow(dead_code)]
pub(crate) fn set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(
    mode: Option<ObservabilityEndpointTlsModeOverride>,
) {
    OBSERVABILITY_ENDPOINT_TLS_MODE_OVERRIDE_FOR_TESTS.with(|tls_mode_override| {
        tls_mode_override.replace(mode);
    });
}

pub(super) fn resolve_observability_endpoint_tls_mode(
) -> Result<ObservabilityEndpointTlsMode, String> {
    if let Some(tls_mode_override) = OBSERVABILITY_ENDPOINT_TLS_MODE_OVERRIDE_FOR_TESTS
        .with(|tls_mode_override| tls_mode_override.borrow().clone())
    {
        return match tls_mode_override {
            ObservabilityEndpointTlsModeOverride::InvalidMode { mode } => Err(format!(
                "observability endpoint tls mode is invalid: {mode} (supported: {OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED},{OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE})"
            )),
            ObservabilityEndpointTlsModeOverride::Require {
                cert_file,
                key_file,
            } => {
                if cert_file.trim().is_empty() {
                    return Err(
                        "observability endpoint tls cert override must not be empty".to_owned()
                    );
                }
                if key_file.trim().is_empty() {
                    return Err(
                        "observability endpoint tls key override must not be empty".to_owned()
                    );
                }
                validate_observability_endpoint_tls_materials(
                    cert_file.as_str(),
                    key_file.as_str(),
                )?;
                Ok(ObservabilityEndpointTlsMode::Require {
                    cert_file,
                    key_file,
                })
            }
        };
    }

    match env::var(OBSERVABILITY_ENDPOINT_TLS_MODE_ENV) {
        Ok(value) => {
            let mode = value.trim().to_ascii_lowercase();
            if mode.is_empty() {
                return Err(format!(
                    "observability endpoint tls mode env must not be empty: {OBSERVABILITY_ENDPOINT_TLS_MODE_ENV}"
                ));
            }
            match mode.as_str() {
                OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED => Ok(ObservabilityEndpointTlsMode::Disabled),
                OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE => {
                    let cert_file = env::var(OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "observability endpoint tls mode requires env: {OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if cert_file.is_empty() {
                        return Err(format!(
                            "observability endpoint tls cert env must not be empty: {OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV}"
                        ));
                    }
                    let key_file = env::var(OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "observability endpoint tls mode requires env: {OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if key_file.is_empty() {
                        return Err(format!(
                            "observability endpoint tls key env must not be empty: {OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV}"
                        ));
                    }
                    validate_observability_endpoint_tls_materials(
                        cert_file.as_str(),
                        key_file.as_str(),
                    )?;
                    Ok(ObservabilityEndpointTlsMode::Require {
                        cert_file,
                        key_file,
                    })
                }
                other => Err(format!(
                    "observability endpoint tls mode is invalid: {other} (supported: {OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED},{OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE})"
                )),
            }
        }
        Err(env::VarError::NotPresent) => Ok(ObservabilityEndpointTlsMode::Disabled),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "observability endpoint tls mode env must be utf-8: {OBSERVABILITY_ENDPOINT_TLS_MODE_ENV}"
        )),
    }
}

fn validate_observability_endpoint_tls_materials(
    cert_file: &str,
    key_file: &str,
) -> Result<(), String> {
    let cert_bytes = fs::read(cert_file).map_err(|error| {
        format!("observability endpoint tls certificate file read failed: {cert_file}: {error}")
    })?;
    let key_bytes = fs::read(key_file).map_err(|error| {
        format!("observability endpoint tls key file read failed: {key_file}: {error}")
    })?;

    let mut cert_reader = BufReader::new(cert_bytes.as_slice());
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "observability endpoint tls certificate file parse failed: {cert_file}: {error}"
            )
        })?;
    if certs.is_empty() {
        return Err(format!(
            "observability endpoint tls certificate file parse failed: {cert_file}: no certificates found"
        ));
    }

    let mut key_reader = BufReader::new(key_bytes.as_slice());
    let private_key = rustls_pemfile::private_key(&mut key_reader).map_err(|error| {
        format!("observability endpoint tls key file parse failed: {key_file}: {error}")
    })?;
    if private_key.is_none() {
        return Err(format!(
            "observability endpoint tls key file parse failed: {key_file}: no private key found"
        ));
    }
    Ok(())
}
