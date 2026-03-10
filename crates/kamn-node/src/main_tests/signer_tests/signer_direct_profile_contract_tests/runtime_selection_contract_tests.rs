use super::super::support::managed_external_core_signer_env_guards;
use super::super::*;
use super::support::runtime_args;

#[test]
fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers() {
    let _lock = lock_signer_env_guard();
    let _env = secondary_runtime_env();
    let rendered = execute_rendered_report(secondary_runtime_args(secondary_runtime_base_url()));
    assert_secondary_rendered(report_view(rendered.as_str()));
}

#[test]
fn integration_runtime_kolme_live_renders_managed_external_signer_selection_markers() {
    let _lock = lock_signer_env_guard();
    let _core_env = managed_external_core_signer_env_guards();
    let _env = managed_external_runtime_env();
    let rendered =
        execute_rendered_report(runtime_args(managed_runtime_base_url(), "managed-external"));
    assert_managed_external_rendered(report_view(rendered.as_str()));
}

fn secondary_runtime_env() -> (EnvVarGuard, EnvVarGuard, EnvVarGuard) {
    let profile = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-secondary"));
    let primary = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let secondary = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
        Some(TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY),
    );
    (profile, primary, secondary)
}

fn secondary_runtime_base_url() -> String {
    spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":37,"account_id":"acct-live-secondary"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:ef56ab78","finality":"final"}"#,
        ),
    ])
    .0
}

fn secondary_runtime_args(base_url: String) -> Vec<String> {
    vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        base_url,
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
        "--output".to_owned(),
        "json".to_owned(),
    ]
}

fn execute_rendered_report(args: Vec<String>) -> String {
    let parsed = parse_args(args).expect("kolme-live args should parse");
    let report = execute(parsed).expect("kolme-live execution should succeed");
    render_bootstrap_report(&report, OutputMode::json())
}

fn report_view(rendered: &str) -> &str {
    rendered
}

fn assert_secondary_rendered(rendered: &str) {
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-secondary\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY\""
    ));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"env-local\""));
}

fn managed_external_runtime_env() -> ManagedExternalRuntimeEnv {
    let profile = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PROFILE", Some("ops-primary"));
    let key_ref = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        Some(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE),
    );
    let primary = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX", None);
    let fallback = EnvVarGuard::set("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None);
    let managed_pubkey = managed_runtime_pubkey();
    let pubkey = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
        Some(managed_pubkey.as_str()),
    );
    let backend = EnvVarGuard::set(
        "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
        Some(managed_runtime_backend_command().as_str()),
    );
    vec![profile, key_ref, primary, fallback, pubkey, backend]
}

fn managed_runtime_base_url() -> String {
    spawn_kolme_live_mock_server(vec![
        MockHttpReply::ok(r#"{"next_nonce":43,"account_id":"acct-live-managed"}"#),
        MockHttpReply::ok(
            r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:aa11bb22","finality":"final"}"#,
        ),
    ])
    .0
}

fn assert_managed_external_rendered(rendered: &str) {
    assert!(rendered.contains("\"kolme_live_signer_profile\":\"ops-primary\""));
    assert!(rendered.contains("\"kolme_live_signer_key_source\":\"managed-external\""));
    assert!(rendered.contains(
        "\"kolme_live_signer_private_key_env\":\"KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX\""
    ));
}

fn managed_runtime_pubkey() -> String {
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    )
}

fn managed_runtime_backend_command() -> String {
    let request = managed_runtime_request();
    let signing_key = build_kolme_live_managed_signing_key(TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE)
        .expect("managed signing key should derive");
    let managed_pubkey = managed_runtime_pubkey();
    let canonical_message =
        render_kolme_live_native_direct_message(&request, managed_pubkey.as_str(), 43)
            .expect("canonical message should render");
    let (backend_signature, backend_recovery_id) = signing_key
        .sign_recoverable(canonical_message.as_bytes())
        .expect("managed signing key should sign canonical message");
    format!(
        "printf 'signature_hex={}\\nrecovery_id={}\\nsigner_public_key_hex={}\\n'",
        encode_kolme_hex_lower(backend_signature.to_bytes().as_ref()),
        backend_recovery_id.to_byte(),
        managed_pubkey,
    )
}

fn managed_runtime_request() -> KolmeRuntimeCommitRequest {
    build_kolme_live_request(
        &bootstrap(NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "./data".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        })
        .expect("bootstrap plan should build"),
    )
    .expect("runtime commit request should build")
}

type ManagedExternalRuntimeEnv = Vec<EnvVarGuard>;
