const SIGNER_POLICY_SRC: &str = include_str!("../src/signer/signer_policy.rs");
const RUNTIME_NETWORK_DOC: &str = include_str!("../../../docs/foundation/runtime-network.md");

const REQUIRED_SIGNER_POLICY_REASON_MARKERS: &[&str] = &[
    "runtime_signer_profile_selector_mismatch",
    "runtime_signer_previous_profile_invalid",
    "runtime_signer_attestation_approved_signers_invalid",
    "runtime_signer_attestation_approved_signers_not_unique",
    "runtime_signer_key_source_profile_pair_disallowed",
    "runtime_signer_rotation_epoch_invalid",
    "runtime_signer_previous_rotation_epoch_invalid",
    "runtime_signer_rotation_epoch_stale",
    "runtime_signer_rotation_epoch_regressed",
    "runtime_signer_attestation_required_approvals_invalid",
    "runtime_signer_failover_attestation_required_approvals_insufficient",
    "runtime_signer_failover_attestation_previous_profile_not_approved",
    "runtime_signer_quorum_linkage_violation",
    "runtime_signer_attestation_quorum_shortfall",
    "managed_signer_key_reference_missing",
    "managed_signer_key_reference_invalid",
    "managed_signer_key_reference_role_invalid",
];

#[test]
fn source_declares_required_signer_policy_reason_markers() {
    for marker in REQUIRED_SIGNER_POLICY_REASON_MARKERS {
        assert!(
            SIGNER_POLICY_SRC.contains(marker),
            "signer_policy.rs missing required reason marker: {marker}"
        );
    }
}

#[test]
fn docs_runtime_network_declares_signer_policy_reason_taxonomy_markers() {
    assert!(
        RUNTIME_NETWORK_DOC.contains("### Signer Policy Reason Taxonomy"),
        "runtime-network docs must declare signer policy taxonomy section"
    );
    assert!(
        RUNTIME_NETWORK_DOC.contains("signer_policy_reason_taxonomy_version=v1"),
        "runtime-network docs must declare signer policy taxonomy version marker"
    );

    for marker in REQUIRED_SIGNER_POLICY_REASON_MARKERS {
        assert!(
            RUNTIME_NETWORK_DOC.contains(marker),
            "runtime-network docs missing signer policy reason marker: {marker}"
        );
    }
}
