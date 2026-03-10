use super::super::*;

#[test]
fn rejects_kolme_live_with_invalid_signing_profile() {
    assert_parse_error(
        with_pairs(
            kolme_live_args(),
            &[
                ("--kolme-live-base-url", "http://127.0.0.1:3000"),
                ("--kolme-live-provider-hint", "kolme-fork-local"),
                ("--kolme-live-signing-profile", "synthetic-signing-profile"),
            ],
        ),
        ConfigError::InvalidKolmeLiveSigningProfile("synthetic-signing-profile".to_owned()),
    );
}

#[test]
fn rejects_kolme_live_with_in_memory_provider_hint_marker() {
    assert_parse_error(
        with_pairs(
            kolme_live_args(),
            &[
                ("--kolme-live-base-url", "http://127.0.0.1:3000"),
                ("--kolme-live-provider-hint", "InMemoryKolmeRuntimeCommitClient"),
                ("--kolme-live-signing-profile", "kolme-fork-secp256k1-v1"),
            ],
        ),
        ConfigError::InvalidKolmeLiveProviderHint("InMemoryKolmeRuntimeCommitClient".to_owned()),
    );
}
