use super::support::test_primary_managed_selection;
use super::{ConfigError, KolmeLiveManagedKeySourceProvenanceMarker};

#[test]
fn regression_managed_key_source_provenance_marker_profile_mismatch_fails_closed() {
    let selection = test_primary_managed_selection();
    let marker = KolmeLiveManagedKeySourceProvenanceMarker {
        profile: "ops-secondary",
        key_source: selection.key_source,
        key_reference_env: selection.key_reference_env,
        signer_public_key_hex: "021111111111111111111111111111111111111111111111111111111111111111"
            .to_owned(),
    };
    let error = super::super::enforce_kolme_live_managed_key_source_provenance_marker_parity(
        &selection, &marker,
    )
    .expect_err("profile mismatch must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("managed_signer_provenance_marker_profile_mismatch"))
    );
}
