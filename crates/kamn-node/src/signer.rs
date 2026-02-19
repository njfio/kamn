use std::env;
#[cfg(test)]
use std::time::{Duration, Instant};

#[cfg(test)]
use kamn_core::KolmeRuntimeCommitProviderError;
use kamn_core::{
    ConfigError, KolmeApiBroadcastRequest, KolmeRuntimeCommitHttpTransport,
    KolmeRuntimeCommitRequest, SignerProviderHandshakeMatrix,
};
use zeroize::Zeroize;

use super::wire_payload::render_kolme_live_native_direct_message;
use super::{
    KOLME_LIVE_SIGNER_KEY_REF_ENV, KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV,
    KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV, KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV, KOLME_LIVE_SIGNER_PROFILE_ENV,
    KOLME_LIVE_SIGNER_PROFILE_PRIMARY, KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
};

mod managed_backend;
mod nonce;
mod signer_adapter;
mod signer_policy;
pub(crate) use managed_backend::{
    resolve_kolme_live_managed_signer_required_marker, sign_kolme_live_managed_external_message,
};
use managed_backend::{
    resolve_required_kolme_live_managed_signer_command,
    resolve_required_managed_signer_public_key_hex,
};
pub(crate) use nonce::resolve_kolme_live_nonce;
#[cfg(test)]
use nonce::{classify_nonce_retry_category, deterministic_nonce_retry_backoff_millis};
#[cfg(test)]
pub(crate) use signer_adapter::{
    build_kolme_live_managed_signing_key, resolve_kolme_live_signer_private_key_env_name,
};
pub(crate) use signer_adapter::{
    decode_kolme_hex_bytes, encode_kolme_hex_lower, KolmeForkSecp256k1SignerAdapter,
};
pub(crate) use signer_policy::{
    evaluate_kolme_live_signer_preflight_readiness, normalize_kolme_live_signer_key_source,
    normalize_kolme_live_signer_profile_selector,
};
use signer_policy::{
    read_required_kolme_live_key_reference_from_env, resolve_kolme_live_signer_selection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KolmeLiveSignerSelection {
    pub(crate) profile: &'static str,
    pub(crate) key_source: &'static str,
    pub(crate) private_key_env: &'static str,
    pub(crate) key_reference_env: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KolmeLiveSignerPreflightReadiness {
    pub(crate) previous_profile: &'static str,
    pub(crate) failover_active: bool,
    pub(crate) rotation_epoch: u64,
    pub(crate) previous_rotation_epoch: u64,
    pub(crate) quorum_linkage_contract_version: &'static str,
    pub(crate) quorum_required_approvals: usize,
    pub(crate) quorum_approved_signers_count: usize,
    pub(crate) quorum_profile_linked: bool,
    pub(crate) quorum_satisfied: bool,
    pub(crate) quorum_linked: bool,
}
pub(crate) fn enforce_kolme_live_signer_preflight(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<KolmeLiveSignerPreflightReadiness, ConfigError> {
    let signer_selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    let readiness = evaluate_kolme_live_signer_preflight_readiness(&signer_selection)?;
    let provider = EnvKolmeLiveSignerSecretProvider;
    provider.ensure_no_fallback_private_key_path()?;
    ensure_kolme_live_strict_signer_secret_source_precedence(
        strict_signer_profile,
        &signer_selection,
    )?;
    if signer_selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        ensure_kolme_live_managed_external_private_key_env_unset(&signer_selection)?;
        let _managed_signer_required_marker = resolve_kolme_live_managed_signer_required_marker()?;
        let _managed_signer_command = resolve_required_kolme_live_managed_signer_command()?;
        let _managed_signer_public_key =
            resolve_required_managed_signer_public_key_hex(&signer_selection)?;
        let _managed_key_reference =
            read_required_kolme_live_key_reference_from_env(&signer_selection)?;
    }
    Ok(readiness)
}

trait KolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError>;

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError>;
}

struct EnvKolmeLiveSignerSecretProvider;

impl KolmeLiveSignerSecretProvider for EnvKolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError> {
        match env::var(KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV) {
            Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must remain unset (fallback_signer_secret_present_violation)"
            ))),
            Err(env::VarError::NotPresent) => Ok(()),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must be valid utf-8 when present (fallback_signer_secret_present_violation)"
            ))),
        }
    }

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError> {
        match env::var(selection.private_key_env) {
            Ok(private_key_hex) => Ok(private_key_hex),
            Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be set for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be valid utf-8 for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
        }
    }
}

fn read_kolme_live_signer_private_key_hex_with_provider<P: KolmeLiveSignerSecretProvider>(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
    provider: &P,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    provider.ensure_no_fallback_private_key_path()?;
    if selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        let key_reference = read_required_kolme_live_key_reference_from_env(&selection)?;
        ensure_kolme_live_managed_external_private_key_env_unset(&selection)?;
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer key reference {key_reference} from {} cannot use private-key signer adapter path; route through managed signer backend execution (managed_signer_private_key_adapter_unsupported)",
            selection.key_reference_env
        )));
    }
    let mut private_key_hex = provider.read_private_key_hex(&selection)?;
    ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
        strict_signer_profile,
        &selection,
        &mut private_key_hex,
    )?;
    Ok((private_key_hex, selection))
}

fn ensure_kolme_live_strict_signer_secret_source_precedence(
    strict_signer_profile: Option<&str>,
    selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    if strict_signer_profile.is_none()
        || selection.key_source != KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL
    {
        return Ok(());
    }
    let non_selected_private_key_env = match selection.profile {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV,
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV,
        _ => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "unsupported signer profile for strict secret-source precedence checks: {} (signer_secret_source_precedence_violation)",
                selection.profile
            )))
        }
    };
    match env::var(non_selected_private_key_env) {
        Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{non_selected_private_key_env} must remain unset when --kolme-live-signer-profile={} and --kolme-live-signer-key-source={} select {} (signer_secret_source_precedence_violation)",
            selection.profile, selection.key_source, selection.private_key_env
        ))),
        Err(env::VarError::NotPresent) => Ok(()),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{non_selected_private_key_env} must be valid utf-8 when present under strict signer source contracts (signer_secret_source_precedence_violation)"
        ))),
    }
}

fn ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
    strict_signer_profile: Option<&str>,
    selection: &KolmeLiveSignerSelection,
    private_key_hex: &mut String,
) -> Result<(), ConfigError> {
    if let Err(error) =
        ensure_kolme_live_strict_signer_secret_source_precedence(strict_signer_profile, selection)
    {
        private_key_hex.zeroize();
        return Err(error);
    }
    Ok(())
}

fn ensure_kolme_live_managed_external_private_key_env_unset(
    selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    match env::var(selection.private_key_env) {
        Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{} must remain unset for signer profile {} when --kolme-live-signer-key-source={} (managed_signer_raw_private_key_forbidden)",
            selection.private_key_env, selection.profile, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{} must be valid utf-8 when present for signer profile {} (managed_signer_raw_private_key_forbidden)",
            selection.private_key_env, selection.profile
        ))),
        Err(env::VarError::NotPresent) => Ok(()),
    }
}

fn read_kolme_live_signer_private_key_hex(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let provider = EnvKolmeLiveSignerSecretProvider;
    read_kolme_live_signer_private_key_hex_with_provider(
        strict_signer_profile,
        strict_signer_key_source,
        &provider,
    )
}

pub(crate) fn build_kolme_live_signer_adapter(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(KolmeForkSecp256k1SignerAdapter, KolmeLiveSignerSelection), ConfigError> {
    let (mut private_key_hex, selection) =
        read_kolme_live_signer_private_key_hex(strict_signer_profile, strict_signer_key_source)?;
    let signer_adapter_result = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
        &mut private_key_hex,
        selection.private_key_env,
    );
    let signer_adapter = signer_adapter_result?;
    Ok((signer_adapter, selection))
}

pub(crate) fn build_kolme_live_direct_signed_wire_payload(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    request: &KolmeRuntimeCommitRequest,
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let signer_selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    let _signer_preflight = evaluate_kolme_live_signer_preflight_readiness(&signer_selection)?;
    let (canonical_message, signature_hex, recovery_id) = if signer_selection.key_source
        == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
    {
        let provider = EnvKolmeLiveSignerSecretProvider;
        provider.ensure_no_fallback_private_key_path()?;
        ensure_kolme_live_managed_external_private_key_env_unset(&signer_selection)?;
        let _managed_signer_required_marker = resolve_kolme_live_managed_signer_required_marker()?;
        let _managed_signer_command = resolve_required_kolme_live_managed_signer_command()?;
        let signer_public_key_hex =
            resolve_required_managed_signer_public_key_hex(&signer_selection)?;
        let key_reference = read_required_kolme_live_key_reference_from_env(&signer_selection)?;
        let nonce = resolve_kolme_live_nonce(base_url, transport, signer_public_key_hex.as_str())?;
        let canonical_message = render_kolme_live_native_direct_message(
            request,
            signer_public_key_hex.as_str(),
            nonce,
        )?;
        let (signature_hex, recovery_id) = sign_kolme_live_managed_external_message(
            key_reference.as_str(),
            request,
            nonce,
            canonical_message.as_str(),
            SignerProviderHandshakeMatrix::with_uniform_availability(true),
            signer_public_key_hex.as_str(),
        )?;
        (canonical_message, signature_hex, recovery_id)
    } else {
        let (signer_adapter, signer_selection_from_adapter) =
            build_kolme_live_signer_adapter(strict_signer_profile, strict_signer_key_source)?;
        let pubkey = signer_adapter.public_key_compressed_hex();
        let nonce = resolve_kolme_live_nonce(base_url, transport, pubkey.as_str())?;
        let canonical_message =
            render_kolme_live_native_direct_message(request, pubkey.as_str(), nonce)?;
        let (signature_hex, recovery_id) =
            signer_adapter.sign_message(canonical_message.as_str())?;
        debug_assert_eq!(
            signer_selection.profile,
            signer_selection_from_adapter.profile
        );
        debug_assert_eq!(
            signer_selection.key_source,
            signer_selection_from_adapter.key_source
        );
        debug_assert_eq!(
            signer_selection.private_key_env,
            signer_selection_from_adapter.private_key_env
        );
        (canonical_message, signature_hex, recovery_id)
    };
    let request = KolmeApiBroadcastRequest::new(
        canonical_message.as_str(),
        signature_hex.as_str(),
        recovery_id,
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    Ok((request.to_json_payload(), signer_selection))
}

#[cfg(test)]
mod tests {
    use super::KolmeForkSecp256k1SignerAdapter;
    use super::{
        classify_nonce_retry_category, deterministic_nonce_retry_backoff_millis,
        evaluate_kolme_live_signer_preflight_readiness, ConfigError, Duration, Instant,
        KolmeLiveSignerSelection, KolmeRuntimeCommitProviderError,
    };
    use std::env;
    use std::sync::{Mutex, OnceLock};

    const TEST_PRIVATE_KEY_HEX: &str =
        "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
    const TEST_PRIVATE_KEY_HEX_SECONDARY: &str =
        "838c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
    const TEST_PRIVATE_KEY_ENV: &str = "TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX";

    fn test_signer_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
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
        }
    }

    fn test_primary_selection() -> KolmeLiveSignerSelection {
        KolmeLiveSignerSelection {
            profile: "ops-primary",
            key_source: "env-local",
            private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        }
    }

    fn is_zeroized_hex_buffer(value: &str) -> bool {
        value.as_bytes().iter().all(|byte| *byte == 0)
    }

    #[test]
    fn regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer() {
        // Regression: #4165
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _secondary_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            Some(TEST_PRIVATE_KEY_HEX_SECONDARY),
        );

        let mut private_key_hex = TEST_PRIVATE_KEY_HEX.to_owned();
        let error = super::ensure_kolme_live_strict_signer_secret_source_precedence_and_zeroize(
            Some("ops-primary"),
            &test_primary_selection(),
            &mut private_key_hex,
        )
        .expect_err("strict signer precedence violation must fail closed");

        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("signer_secret_source_precedence_violation")),
            "strict signer precedence violation must preserve deterministic reason marker"
        );
        assert!(
            is_zeroized_hex_buffer(private_key_hex.as_str()),
            "strict signer precedence violation must scrub env-secret private key buffers"
        );
    }

    #[test]
    fn unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material() {
        const SIGNER_ADAPTER_SOURCE: &str = include_str!("signer/signer_adapter.rs");
        assert!(
            SIGNER_ADAPTER_SOURCE.contains("key_material.zeroize()"),
            "managed signing key transient key material must be explicitly zeroized after key construction"
        );
    }

    #[test]
    fn unit_signer_private_key_parse_zeroizes_hex_buffer_on_success() {
        let mut private_key_hex = TEST_PRIVATE_KEY_HEX.to_owned();
        let signer = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
            &mut private_key_hex,
            TEST_PRIVATE_KEY_ENV,
        )
        .expect("valid private key should parse");
        assert!(
            is_zeroized_hex_buffer(private_key_hex.as_str()),
            "private key hex buffer must be scrubbed after successful signer construction"
        );
        assert_eq!(signer.private_key_env, TEST_PRIVATE_KEY_ENV);
    }

    #[test]
    fn regression_signer_private_key_parse_zeroizes_hex_buffer_on_failure() {
        // Regression: #2672
        let mut private_key_hex = "zz".to_owned();
        let error = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
            &mut private_key_hex,
            TEST_PRIVATE_KEY_ENV,
        )
        .expect_err("invalid private key hex must fail closed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("invalid hex character")),
            "invalid hex decode must fail with deterministic validation error"
        );
        assert!(
            is_zeroized_hex_buffer(private_key_hex.as_str()),
            "private key hex buffer must be scrubbed after parse failure"
        );
    }

    #[test]
    fn regression_signer_module_source_contains_no_unreachable_macro() {
        // Regression: #3941
        const SIGNER_SOURCE: &str = include_str!("signer.rs");
        let marker = ["unreachable", "!", "("].concat();
        assert!(
            !SIGNER_SOURCE.contains(marker.as_str()),
            "signer source must remain unreachable-macro free"
        );
    }

    #[test]
    fn regression_signer_private_key_decode_failure_redacts_sensitive_input() {
        // Regression: #3914
        let sensitive_input = "secretshouldnotappear000";
        let mut private_key_hex = sensitive_input.to_owned();
        let error = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
            &mut private_key_hex,
            TEST_PRIVATE_KEY_ENV,
        )
        .expect_err("invalid private key material must fail closed");
        assert!(
            matches!(&error, ConfigError::RuntimeKolmeLive(_)),
            "decode failure should map to RuntimeKolmeLive: {error:?}"
        );
        let message = match &error {
            ConfigError::RuntimeKolmeLive(message) => message,
            _ => "",
        };
        assert!(
            message.contains("invalid hex character"),
            "decode failure must remain deterministic"
        );
        assert!(
            !message.contains(sensitive_input),
            "decode failure must not echo raw private key input"
        );
        assert!(
            is_zeroized_hex_buffer(private_key_hex.as_str()),
            "private key hex buffer must be scrubbed after decode failure"
        );
    }

    #[test]
    fn unit_nonce_retry_classifier_marks_transient_provider_errors() {
        assert_eq!(
            classify_nonce_retry_category(&KolmeRuntimeCommitProviderError::Timeout),
            Some("timeout")
        );
        assert_eq!(
            classify_nonce_retry_category(&KolmeRuntimeCommitProviderError::Unavailable {
                reason: "network unavailable".to_owned(),
            }),
            Some("unavailable")
        );
        assert_eq!(
            classify_nonce_retry_category(&KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "missing next_nonce".to_owned(),
            }),
            None
        );
    }

    #[test]
    fn unit_nonce_retry_backoff_policy_is_deterministic_and_bounded() {
        assert_eq!(deterministic_nonce_retry_backoff_millis(1), 10);
        assert_eq!(deterministic_nonce_retry_backoff_millis(2), 20);
        assert_eq!(deterministic_nonce_retry_backoff_millis(3), 40);
        assert_eq!(deterministic_nonce_retry_backoff_millis(8), 40);
    }

    #[test]
    fn performance_signer_private_key_parse_zeroization_stays_bounded() {
        let started = Instant::now();
        for _ in 0..2_000 {
            let mut private_key_hex = TEST_PRIVATE_KEY_HEX.to_owned();
            let _signer = KolmeForkSecp256k1SignerAdapter::from_private_key_hex_in_place(
                &mut private_key_hex,
                TEST_PRIVATE_KEY_ENV,
            )
            .expect("valid private key should parse");
            assert!(
                is_zeroized_hex_buffer(private_key_hex.as_str()),
                "private key buffer should remain scrubbed during parse loop"
            );
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "private key parse + zeroization loop exceeded 2s for 2k iterations"
        );
    }

    #[test]
    fn regression_strict_signer_secret_source_precedence_rejects_dual_private_key_envs() {
        // Regression: #4660
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
        let _primary_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_PRIVATE_KEY_HEX),
        );
        let _secondary_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            Some(TEST_PRIVATE_KEY_HEX_SECONDARY),
        );
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);

        let error =
            super::read_kolme_live_signer_private_key_hex(Some("ops-primary"), Some("env-local"))
                .expect_err("strict signer contracts must reject dual private key env sources");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("signer_secret_source_precedence_violation")),
            "strict signer contracts must fail closed with precedence violation reason"
        );
    }

    #[test]
    fn regression_strict_secondary_profile_requires_secondary_secret_even_with_primary_present() {
        // Regression: #4660
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _profile_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
        let _primary_guard = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
            Some(TEST_PRIVATE_KEY_HEX),
        );
        let _secondary_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY", None);
        let _fallback_guard =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);

        let error =
            super::read_kolme_live_signer_private_key_hex(Some("ops-secondary"), Some("env-local"))
                .expect_err(
                    "strict secondary signer contracts must require secondary private key env",
                );
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY must be set")),
            "secondary strict profile must not bypass selected-secret requirement via primary key env"
        );
    }

    #[test]
    fn unit_signer_preflight_defaults_to_single_signer_quorum_ready() {
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _previous_profile = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE", None);
        let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", None);
        let _previous_rotation_epoch =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", None);
        let _required_approvals =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS", None);
        let _approved_signers =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS", None);

        let readiness = evaluate_kolme_live_signer_preflight_readiness(&test_primary_selection())
            .expect("default single-signer preflight should be ready");
        assert_eq!(readiness.previous_profile, "ops-primary");
        assert!(!readiness.failover_active);
        assert_eq!(readiness.rotation_epoch, 1);
        assert_eq!(readiness.previous_rotation_epoch, 1);
        assert_eq!(readiness.quorum_linkage_contract_version, "v1");
        assert_eq!(readiness.quorum_required_approvals, 1);
        assert_eq!(readiness.quorum_approved_signers_count, 1);
        assert!(readiness.quorum_profile_linked);
        assert!(readiness.quorum_satisfied);
        assert!(readiness.quorum_linked);
    }

    #[test]
    fn regression_signer_preflight_rejects_stale_failover_rotation_epoch() {
        // Regression: #3472
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _previous_profile = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
            Some("ops-primary"),
        );
        let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("2"));
        let _previous_rotation_epoch =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("2"));
        let _required_approvals = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
            Some("2"),
        );
        let _approved_signers = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
            Some("ops-primary,ops-secondary"),
        );
        let selection = KolmeLiveSignerSelection {
            profile: "ops-secondary",
            key_source: "env-local",
            private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
        };
        let error = evaluate_kolme_live_signer_preflight_readiness(&selection)
            .expect_err("stale failover rotation epoch must fail closed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_rotation_epoch_stale")),
            "stale failover rotation epoch must preserve runtime_signer_rotation_epoch_stale"
        );
    }

    #[test]
    fn regression_signer_preflight_rejects_disallowed_secondary_managed_external_pair() {
        // Regression: #3472
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _previous_profile = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
            Some("ops-secondary"),
        );
        let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
        let _previous_rotation_epoch =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("1"));
        let _required_approvals = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
            Some("1"),
        );
        let _approved_signers = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
            Some("ops-secondary"),
        );
        let selection = KolmeLiveSignerSelection {
            profile: "ops-secondary",
            key_source: "managed-external",
            private_key_env: "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
            key_reference_env: "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
        };
        let error = evaluate_kolme_live_signer_preflight_readiness(&selection)
            .expect_err("secondary managed-external pair must fail closed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_key_source_profile_pair_disallowed")),
            "disallowed secondary managed-external pair must preserve reason code"
        );
    }

    #[test]
    fn functional_signer_preflight_rejects_quorum_shortfall() {
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _previous_profile = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
            Some("ops-primary"),
        );
        let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
        let _previous_rotation_epoch =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("1"));
        let _required_approvals = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
            Some("2"),
        );
        let _approved_signers = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
            Some("ops-primary"),
        );
        let error = evaluate_kolme_live_signer_preflight_readiness(&test_primary_selection())
            .expect_err("quorum shortfall must fail closed");
        assert!(
            matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("runtime_signer_attestation_quorum_shortfall")),
            "quorum shortfall must preserve runtime_signer_attestation_quorum_shortfall"
        );
    }

    #[test]
    fn performance_signer_preflight_readiness_stays_bounded() {
        let _lock = test_signer_env_lock()
            .lock()
            .expect("signer env lock should guard test mutation");
        let _previous_profile = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_PREVIOUS_PROFILE",
            Some("ops-primary"),
        );
        let _rotation_epoch = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH", Some("1"));
        let _previous_rotation_epoch =
            EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PREVIOUS_ROTATION_EPOCH", Some("1"));
        let _required_approvals = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS",
            Some("1"),
        );
        let _approved_signers = EnvVarGuard::set(
            "KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS",
            Some("ops-primary"),
        );
        let selection = test_primary_selection();
        let started = Instant::now();
        for _ in 0..5_000 {
            let readiness = evaluate_kolme_live_signer_preflight_readiness(&selection)
                .expect("preflight readiness must remain stable");
            assert!(readiness.quorum_linked);
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "signer preflight readiness exceeded 2s for 5k evaluations"
        );
    }
}
