use super::super::*;

#[test]
fn rejects_kolme_live_without_base_url() {
    let args = with_pairs(
        kolme_live_args(),
        &[
            ("--kolme-live-provider-hint", "kolme-fork-local"),
            ("--kolme-live-signing-profile", "kolme-fork-secp256k1-v1"),
        ],
    );
    assert_parse_error(args, missing_arg("--kolme-live-base-url"));
}

#[test]
fn rejects_kolme_live_without_provider_hint() {
    let args = with_pairs(
        kolme_live_args(),
        &[
            ("--kolme-live-base-url", "http://127.0.0.1:3000"),
            ("--kolme-live-signing-profile", "kolme-fork-secp256k1-v1"),
        ],
    );
    assert_parse_error(args, missing_arg("--kolme-live-provider-hint"));
}

#[test]
fn rejects_kolme_live_without_signing_profile() {
    let args = with_pairs(
        kolme_live_args(),
        &[
            ("--kolme-live-base-url", "http://127.0.0.1:3000"),
            ("--kolme-live-provider-hint", "kolme-fork-local"),
        ],
    );
    assert_parse_error(args, missing_arg("--kolme-live-signing-profile"));
}

#[test]
fn rejects_kolme_live_without_signer_key_source() {
    let args = kolme_live_declared_args();
    assert_parse_error(args, missing_arg("--kolme-live-signer-key-source"));
}

#[test]
fn rejects_kolme_live_continuous_mode_without_tick_interval() {
    let args = with_pairs(
        kolme_live_declared_args(),
        &[("--kolme-live-signer-key-source", "env-local"), ("--daemon-max-ticks", "2")],
    );
    assert_parse_error(args, missing_arg("--daemon-tick-interval-ms"));
}

#[test]
fn rejects_kolme_live_continuous_mode_without_max_ticks() {
    let args = with_pairs(
        kolme_live_declared_args(),
        &[
            ("--kolme-live-signer-key-source", "env-local"),
            ("--daemon-tick-interval-ms", "25"),
        ],
    );
    assert_parse_error(args, missing_arg("--daemon-max-ticks"));
}
