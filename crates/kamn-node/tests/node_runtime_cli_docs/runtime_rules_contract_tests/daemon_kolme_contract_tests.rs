use crate::support::{assert_doc_markers, DOC};

const DAEMON_RULE_MARKERS: &[&str] = &[
    "## Daemon Runtime Rules",
    "--daemon-max-ticks",
    "--daemon-tick-interval-ms",
    "--daemon-shutdown-signal-tick",
    "--daemon-shutdown-drain-ticks",
    "--daemon-shutdown-timeout-ticks",
    "--daemon-lifecycle-event",
    "active construct-lock lease owner",
    "execute_processor_daemon_tick",
    "typed construct-lock errors",
    "tick-budget-exhausted",
    "graceful-shutdown:",
    "graceful-shutdown-timeout:",
    "ignored_signals",
];
const DAEMON_DRAIN_MARKERS: &[&str] = &[
    "shutdown_drain_status",
    "shutdown_signal_tick",
    "shutdown_drain_ticks",
    "shutdown_timeout_ticks",
    "shutdown_ignored_signals",
];
const KOLME_LIVE_MARKERS: &[&str] = &[
    "## Kolme Live Runtime Rules",
    "`kolme-live`",
    "--kolme-live-base-url",
    "--kolme-live-provider-hint",
    "--kolme-live-signing-profile",
    "KolmeRuntimeCommitLiveProvider",
    "kolme-fork-secp256k1-v1",
    "signer-selection evidence markers",
    "KAMN_KOLME_LIVE_SIGNER_PROFILE",
    "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND",
    "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED",
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY",
    "ops-primary",
    "ops-secondary",
    "env-local",
    "managed_signer_backend_required_missing",
    "managed_signer_backend_required_invalid",
    "managed_signer_public_key_marker_missing",
    "managed_signer_public_key_marker_invalid",
    "production_signer_key_source_env_local_forbidden",
    "fallback_signer_secret_present_violation",
    "managed_signer_raw_private_key_forbidden",
    "KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true",
    "signer key-source provenance matrix",
    "runtime must not silently fall back to `env-local`",
    "/runtime-commit/status",
    "max-attempt budget `2`",
    "finality-polled",
    "finality-unavailable",
    "kolme_live_observability_latency_p50_ms",
    "kolme_live_observability_health",
];

#[test]
fn doc_contains_runtime_daemon_rules() {
    assert_doc_markers(DOC, DAEMON_RULE_MARKERS, "node runtime CLI daemon rules");
}

#[test]
fn doc_contains_daemon_shutdown_drain_marker_fields() {
    assert_doc_markers(
        DOC,
        DAEMON_DRAIN_MARKERS,
        "node runtime CLI daemon drain markers",
    );
}

#[test]
fn doc_contains_runtime_kolme_live_rules() {
    assert_doc_markers(DOC, KOLME_LIVE_MARKERS, "node runtime CLI kolme-live rules");
}
