use kamn_core::ConfigError;
use kamn_core::KolmeRuntimeCommitRequest;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KolmeLiveManagedKeySourceProvenanceMarker {
    pub(crate) profile: &'static str,
    pub(crate) key_source: &'static str,
    pub(crate) key_reference_env: &'static str,
    pub(crate) signer_public_key_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KolmeLiveManagedKeySourceAdapterOutput {
    pub(crate) signature_hex: String,
    pub(crate) recovery_id: u8,
    pub(crate) provenance_marker: KolmeLiveManagedKeySourceProvenanceMarker,
}

pub(crate) trait KolmeLiveManagedKeySourceAdapter {
    fn sign_message(
        &self,
        signer_selection: &KolmeLiveSignerSelection,
        key_reference: &str,
        request: &KolmeRuntimeCommitRequest,
        nonce: u64,
        canonical_message: &str,
        signer_public_key_hex: &str,
    ) -> Result<KolmeLiveManagedKeySourceAdapterOutput, ConfigError>;
}

pub(crate) trait KolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError>;

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError>;
}
