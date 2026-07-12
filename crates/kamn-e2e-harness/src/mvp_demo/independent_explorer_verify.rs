use std::path::PathBuf;

use super::settlement_evidence_artifact::SettlementEvidenceArtifact;

const EXPLORER_INVALID: &str = "EXPLORER_LINK_INVALID";
const EXPLORER_PREFIX: &str = "https://explorer.solana.com/tx/";

pub(super) fn validate_explorer_links(
    paths: [PathBuf; 2],
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    let expected = format!(
        "{EXPLORER_PREFIX}{}?cluster=devnet",
        evidence.settlement_tx_signature
    );
    for path in paths {
        let markdown = std::fs::read_to_string(path).map_err(|_| invalid())?;
        if !has_only_expected_links(markdown.as_str(), expected.as_str()) {
            return Err(invalid());
        }
    }
    Ok(())
}

fn has_only_expected_links(markdown: &str, expected: &str) -> bool {
    markdown.contains(expected)
        && markdown
            .match_indices(EXPLORER_PREFIX)
            .all(|(index, _)| markdown[index..].starts_with(expected))
}

fn invalid() -> String {
    EXPLORER_INVALID.to_owned()
}
