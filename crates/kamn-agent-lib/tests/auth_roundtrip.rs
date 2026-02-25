use kamn_agent_lib::auth::KamnAuthHeaders;
use kamn_agent_lib::identity::AgentIdentity;
use kamn_agent_lib::AgentLibError;
use kamn_sdk::service_signature_for_fields;
use std::ffi::OsString;
use std::sync::{Mutex, OnceLock, PoisonError};

const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_env_vars<F>(updates: &[(&str, Option<&str>)], test: F)
where
    F: FnOnce(),
{
    let _guard = env_lock().lock().unwrap_or_else(PoisonError::into_inner);
    let previous = updates
        .iter()
        .map(|(key, _)| ((*key).to_owned(), std::env::var_os(key)))
        .collect::<Vec<(String, Option<OsString>)>>();

    for (key, value) in updates {
        match value {
            Some(value) => {
                // SAFETY: environment mutations are serialized by a process-wide mutex.
                unsafe { std::env::set_var(key, value) }
            }
            None => {
                // SAFETY: environment mutations are serialized by a process-wide mutex.
                unsafe { std::env::remove_var(key) }
            }
        }
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
    for (key, value) in previous {
        match value {
            Some(value) => {
                // SAFETY: environment mutations are serialized by a process-wide mutex.
                unsafe { std::env::set_var(key, value) }
            }
            None => {
                // SAFETY: environment mutations are serialized by a process-wide mutex.
                unsafe { std::env::remove_var(key) }
            }
        }
    }
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

#[test]
fn spec_c01_auth_roundtrip_signature_matches_service_crypto_contract() {
    let identity = AgentIdentity::from_did_and_signing_key(
        "kamn:did:agent:alice",
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("identity");
    let body = r#"{"message":"hello"}"#;
    let headers = KamnAuthHeaders::build(
        identity.did().as_str(),
        identity.signing_key(),
        7,
        "service-api:kamn-devnet:v0.1.0",
        body.as_bytes(),
        Some("messages:write"),
    )
    .expect("headers");
    let expected_signature = with_env_signature(
        identity.did(),
        7,
        "kamn-devnet",
        "v0.1.0",
        body,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );

    assert_eq!(headers.sender_did_header, identity.did().as_str());
    assert_eq!(headers.nonce_header, "7");
    assert_eq!(
        headers.authz_scope_header.as_deref(),
        Some("messages:write")
    );
    assert_eq!(headers.signature_header, expected_signature);
}

#[test]
fn spec_c02_auth_roundtrip_rejects_non_private_key_signing_material() {
    let error = KamnAuthHeaders::build(
        "kamn:did:agent:alice",
        "ed25519:alice:signing",
        5,
        "service-api:kamn-devnet:v0.1.0",
        br#"{"message":"hello"}"#,
        Some("messages:write"),
    )
    .expect_err("non-hex signing material must fail closed");
    assert_eq!(
        error,
        AgentLibError::InvalidInput {
            field: "signing_key",
            reason: "must be valid secp256k1 private key hex".to_owned(),
        }
    );
}

#[test]
fn spec_c03_auth_roundtrip_forged_deterministic_signature_is_rejected() {
    let identity = AgentIdentity::from_did_and_signing_key(
        "kamn:did:agent:alice",
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("identity");
    let body = r#"{"message":"hello"}"#;
    let headers = KamnAuthHeaders::build(
        identity.did().as_str(),
        identity.signing_key(),
        9,
        "service-api:kamn-devnet:v0.1.0",
        body.as_bytes(),
        Some("messages:write"),
    )
    .expect("headers");
    let forged_signature = format!(
        "sig:deterministic-v1:baseline-v1:{}:{}:{}:{}",
        identity.did().as_str(),
        9,
        "service-api:kamn-devnet:v0.1.0",
        body.len()
    );

    assert_ne!(headers.signature_header, forged_signature);
}

#[test]
fn spec_c04_auth_roundtrip_tampered_same_length_payload_changes_signature() {
    let identity = AgentIdentity::from_did_and_signing_key(
        "kamn:did:agent:alice",
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("identity");
    let body_a = r#"{"message":"hello"}"#;
    let body_b = r#"{"message":"jello"}"#;
    let headers_a = KamnAuthHeaders::build(
        identity.did().as_str(),
        identity.signing_key(),
        11,
        "service-api:kamn-devnet:v0.1.0",
        body_a.as_bytes(),
        Some("messages:write"),
    )
    .expect("headers");
    let headers_b = KamnAuthHeaders::build(
        identity.did().as_str(),
        identity.signing_key(),
        11,
        "service-api:kamn-devnet:v0.1.0",
        body_b.as_bytes(),
        Some("messages:write"),
    )
    .expect("headers");

    assert_eq!(body_a.len(), body_b.len(), "fixture requires equal lengths");
    assert_ne!(headers_a.signature_header, headers_b.signature_header);
}

fn with_env_signature(
    sender_did: &kamn_sdk::AgentDid,
    nonce: u64,
    chain_id: &str,
    chain_version: &str,
    body: &str,
    private_key_hex: &str,
) -> String {
    let mut signature = String::new();
    with_env_vars(
        &[(
            "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
            Some(private_key_hex),
        )],
        || {
            signature =
                service_signature_for_fields(sender_did, nonce, chain_id, chain_version, body)
                    .expect("service signature");
        },
    );
    signature
}
