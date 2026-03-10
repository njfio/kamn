use super::super::super::support::*;

#[test]
fn spec_c05_fallback_retirement_docs_parity_markers() {
    let docs = fallback_retirement_docs();
    for marker in fallback_retirement_markers() {
        assert_marker_present_in_all_docs(&docs, marker);
    }
}

fn fallback_retirement_docs() -> [String; 3] {
    [
        read_text("docs/developer/readme-contract-reference.md"),
        read_text("docs/ci/strategy.md"),
        read_text("docs/planning/kolme-devnet-ops.md"),
    ]
}

fn fallback_retirement_markers() -> [&'static str; 8] {
    [
        "fallback_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_rejected_profile_class=production",
        "contracts.fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract",
        "fallback_signer_secret_present_violation",
        "fallback_signer_secret_checkpoint_reason_mismatch",
        "Regression: #2337",
    ]
}

fn assert_marker_present_in_all_docs(docs: &[String; 3], marker: &str) {
    for (doc_index, doc) in docs.iter().enumerate() {
        assert!(
            doc.contains(marker),
            "fallback marker missing from doc #{doc_index}: {marker}"
        );
    }
}
