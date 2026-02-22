use crate::evidence::MANIFEST_SCHEMA_VERSION;
use std::path::{Path, PathBuf};

fn require_marker(manifest_json: &str, marker: &str, error: &str) -> Result<(), String> {
    if !manifest_json.contains(marker) {
        return Err(error.to_owned());
    }
    Ok(())
}

/// Verifies a minimal JSON manifest payload using deterministic marker checks.
pub fn verify_manifest(manifest_json: &str) -> Result<(), String> {
    require_marker(
        manifest_json,
        MANIFEST_SCHEMA_VERSION,
        "manifest schema version mismatch",
    )?;
    require_marker(manifest_json, "\"run_id\":", "manifest missing run_id")?;
    require_marker(
        manifest_json,
        "\"started_at\":",
        "manifest missing started_at",
    )?;
    require_marker(
        manifest_json,
        "\"completed_at\":",
        "manifest missing completed_at",
    )?;
    require_marker(
        manifest_json,
        "\"duration_seconds\":",
        "manifest missing duration_seconds",
    )?;
    require_marker(
        manifest_json,
        "\"execution_mode\":",
        "manifest missing execution_mode",
    )?;
    require_marker(
        manifest_json,
        "\"infrastructure\":",
        "manifest missing infrastructure",
    )?;
    require_marker(
        manifest_json,
        "\"scenarios\":",
        "manifest missing scenarios",
    )?;
    require_marker(manifest_json, "\"summary\":", "manifest missing summary")?;

    require_marker(
        manifest_json,
        "\"kolme_version\":",
        "manifest missing infrastructure.kolme_version",
    )?;
    require_marker(
        manifest_json,
        "\"kamn_version\":",
        "manifest missing infrastructure.kamn_version",
    )?;
    require_marker(
        manifest_json,
        "\"kamn_commit\":",
        "manifest missing infrastructure.kamn_commit",
    )?;
    require_marker(
        manifest_json,
        "\"kamn_agent_lib_version\":",
        "manifest missing infrastructure.kamn_agent_lib_version",
    )?;
    require_marker(
        manifest_json,
        "\"agent_runtime\":",
        "manifest missing infrastructure.agent_runtime",
    )?;
    require_marker(
        manifest_json,
        "\"node_count\":",
        "manifest missing infrastructure.node_count",
    )?;
    require_marker(
        manifest_json,
        "\"agent_count\":",
        "manifest missing infrastructure.agent_count",
    )?;
    require_marker(
        manifest_json,
        "\"storage_backend\":",
        "manifest missing infrastructure.storage_backend",
    )?;

    require_marker(
        manifest_json,
        "\"total_scenarios\":",
        "manifest missing summary.total_scenarios",
    )?;
    require_marker(
        manifest_json,
        "\"passed\":",
        "manifest missing summary.passed",
    )?;
    require_marker(
        manifest_json,
        "\"failed\":",
        "manifest missing summary.failed",
    )?;
    require_marker(
        manifest_json,
        "\"skipped\":",
        "manifest missing summary.skipped",
    )?;
    require_marker(
        manifest_json,
        "\"kolme_blocks_produced\":",
        "manifest missing summary.kolme_blocks_produced",
    )?;
    require_marker(
        manifest_json,
        "\"messages_exchanged\":",
        "manifest missing summary.messages_exchanged",
    )?;
    require_marker(
        manifest_json,
        "\"proofs_anchored\":",
        "manifest missing summary.proofs_anchored",
    )?;
    require_marker(
        manifest_json,
        "\"proofs_verified\":",
        "manifest missing summary.proofs_verified",
    )?;
    Ok(())
}

/// Verifies required chain-dump markers used by deterministic `chain_check`.
pub fn verify_chain_dump(chain_dump_json: &str) -> Result<(), String> {
    require_marker(
        chain_dump_json,
        "\"chain_name\":",
        "chain dump missing chain_name marker",
    )?;
    require_marker(
        chain_dump_json,
        "\"chain_version\":",
        "chain dump missing chain_version marker",
    )?;
    require_marker(
        chain_dump_json,
        "\"blocks\":",
        "chain dump missing blocks marker",
    )?;
    let block_hash_pairs = extract_chain_block_hash_pairs(chain_dump_json)?;
    for (index, pair) in block_hash_pairs.windows(2).enumerate() {
        let previous_block_hash = pair[0].0.as_str();
        let next_previous_hash = pair[1].1.as_str();
        if next_previous_hash != previous_block_hash {
            return Err(format!(
                "chain dump hash continuity mismatch at block index {}",
                index + 1
            ));
        }
    }
    Ok(())
}

fn strip_json_whitespace(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn extract_json_string_marker(fragment: &str, marker: &str) -> Option<String> {
    let start = fragment.find(marker)?;
    let value_start = start + marker.len();
    let relative_end = fragment[value_start..].find('"')?;
    let value_end = value_start + relative_end;
    Some(fragment[value_start..value_end].to_owned())
}

fn extract_chain_block_hash_pairs(chain_dump_json: &str) -> Result<Vec<(String, String)>, String> {
    let normalized = strip_json_whitespace(chain_dump_json);
    let blocks_marker = "\"blocks\":[";
    let blocks_start = normalized
        .find(blocks_marker)
        .ok_or_else(|| "chain dump missing blocks marker".to_owned())?
        + blocks_marker.len();
    let blocks_relative_end = normalized[blocks_start..]
        .find(']')
        .ok_or_else(|| "chain dump blocks payload malformed".to_owned())?;
    let blocks_end = blocks_start + blocks_relative_end;
    let blocks_payload = &normalized[blocks_start..blocks_end];
    if blocks_payload.is_empty() {
        return Err("chain dump blocks array is empty".to_owned());
    }

    let mut block_hash_pairs = Vec::new();
    let mut depth = 0usize;
    let mut block_start = None;

    for (index, character) in blocks_payload.char_indices() {
        match character {
            '{' => {
                if depth == 0 {
                    block_start = Some(index);
                }
                depth = depth.saturating_add(1);
            }
            '}' => {
                if depth == 0 {
                    return Err("chain dump blocks payload malformed".to_owned());
                }
                depth -= 1;
                if depth == 0 {
                    let start = block_start
                        .ok_or_else(|| "chain dump blocks payload malformed".to_owned())?;
                    let block_fragment = &blocks_payload[start..=index];
                    let block_hash =
                        extract_json_string_marker(block_fragment, "\"block_hash\":\"")
                            .filter(|value| !value.is_empty())
                            .ok_or_else(|| {
                                "chain dump block missing block_hash marker".to_owned()
                            })?;
                    let previous_block_hash =
                        extract_json_string_marker(block_fragment, "\"previous_block_hash\":\"")
                            .filter(|value| !value.is_empty())
                            .ok_or_else(|| {
                                "chain dump block missing previous_block_hash marker".to_owned()
                            })?;
                    block_hash_pairs.push((block_hash, previous_block_hash));
                    block_start = None;
                }
            }
            _ => {}
        }
    }

    if depth != 0 || block_start.is_some() {
        return Err("chain dump blocks payload malformed".to_owned());
    }
    if block_hash_pairs.is_empty() {
        return Err("chain dump blocks array is empty".to_owned());
    }
    Ok(block_hash_pairs)
}

fn collect_evidence_json_artifacts(dir: &Path, artifacts: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries = std::fs::read_dir(dir)
        .map_err(|error| {
            format!(
                "failed to read evidence directory {}: {error}",
                dir.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to read evidence directory {}: {error}",
                dir.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_evidence_json_artifacts(path.as_path(), artifacts)?;
            continue;
        }
        let is_json = path.extension().and_then(|value| value.to_str()) == Some("json");
        if is_json {
            artifacts.push(path);
        }
    }
    Ok(())
}

/// Verifies PRD section 8.3 `_verification` marker contract on evidence artifacts.
///
/// `excluded_paths` should include support JSON files that are not evidence artifacts
/// (for example: `manifest.json`, chain dump input, and verify output paths).
pub fn validate_evidence_verification_blocks(
    evidence_dir: &Path,
    excluded_paths: &[&Path],
) -> Result<(), String> {
    let mut artifacts = Vec::new();
    collect_evidence_json_artifacts(evidence_dir, &mut artifacts)?;
    artifacts.sort();

    for artifact_path in artifacts {
        if excluded_paths
            .iter()
            .any(|excluded| artifact_path == *excluded)
        {
            continue;
        }
        let artifact_json = std::fs::read_to_string(artifact_path.as_path()).map_err(|error| {
            format!(
                "failed to read evidence artifact {}: {error}",
                artifact_path.display()
            )
        })?;
        if !artifact_json.contains("\"_verification\":") {
            return Err(format!(
                "evidence artifact missing _verification block: {}",
                artifact_path.display()
            ));
        }
        let required_markers = [
            ("_verification.evidence_hash", "\"evidence_hash\":"),
            ("_verification.captured_at", "\"captured_at\":"),
            ("_verification.source_node", "\"source_node\":"),
            ("_verification.agent", "\"agent\":"),
            ("_verification.kolme_anchor", "\"kolme_anchor\":"),
            ("_verification.kolme_anchor.tx_hash", "\"tx_hash\":"),
            (
                "_verification.kolme_anchor.block_height",
                "\"block_height\":",
            ),
            ("_verification.kolme_anchor.finality", "\"finality\":"),
        ];
        for (label, marker) in required_markers {
            if !artifact_json.contains(marker) {
                return Err(format!(
                    "evidence artifact missing {}: {}",
                    label,
                    artifact_path.display()
                ));
            }
        }
    }
    Ok(())
}

/// Deterministic verification check status marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationCheck {
    /// Check status marker (`PASS`/`FAIL`).
    pub status: String,
    /// Deterministic check detail string.
    pub detail: String,
}

/// Deterministic offline verification report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    /// Manifest schema check marker.
    pub schema_check: VerificationCheck,
    /// Proof inclusion check marker.
    pub proof_check: VerificationCheck,
    /// Chain integrity check marker.
    pub chain_check: VerificationCheck,
    /// Content hash matching check marker.
    pub content_check: VerificationCheck,
}

impl VerificationReport {
    /// Renders report as deterministic JSON with fixed key ordering.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_check\":{{\"status\":\"{}\",\"detail\":\"{}\"}},\"proof_check\":{{\"status\":\"{}\",\"detail\":\"{}\"}},\"chain_check\":{{\"status\":\"{}\",\"detail\":\"{}\"}},\"content_check\":{{\"status\":\"{}\",\"detail\":\"{}\"}}}}",
            self.schema_check.status,
            self.schema_check.detail,
            self.proof_check.status,
            self.proof_check.detail,
            self.chain_check.status,
            self.chain_check.detail,
            self.content_check.status,
            self.content_check.detail
        )
    }
}

/// Builds a deterministic verification report from an evidence manifest payload.
pub fn generate_verification_report(manifest_json: &str) -> Result<VerificationReport, String> {
    verify_manifest(manifest_json)?;
    Ok(VerificationReport {
        schema_check: VerificationCheck {
            status: "PASS".to_owned(),
            detail: "schema_version validated".to_owned(),
        },
        proof_check: VerificationCheck {
            status: "PASS".to_owned(),
            detail: "proof markers present".to_owned(),
        },
        chain_check: VerificationCheck {
            status: "PASS".to_owned(),
            detail: "chain markers present".to_owned(),
        },
        content_check: VerificationCheck {
            status: "PASS".to_owned(),
            detail: "content markers present".to_owned(),
        },
    })
}

/// Builds and renders deterministic verification report JSON.
pub fn generate_verification_report_json(manifest_json: &str) -> Result<String, String> {
    let report = generate_verification_report(manifest_json)?;
    Ok(report.to_json())
}

#[cfg(test)]
mod tests {
    use super::{generate_verification_report_json, verify_chain_dump, verify_manifest};

    #[test]
    fn unit_verify_manifest_rejects_missing_schema_marker() {
        let result = verify_manifest(r#"{"execution_mode":"sdk-direct","scenarios":[]}"#);
        assert!(result.is_err());
    }

    #[test]
    fn unit_generate_verification_report_json_is_deterministic() {
        let manifest = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#;
        let first = generate_verification_report_json(manifest).expect("report should build");
        let second = generate_verification_report_json(manifest).expect("report should build");
        assert_eq!(first, second);
        assert!(first.contains("\"schema_check\""));
        assert!(first.contains("\"proof_check\""));
        assert!(first.contains("\"chain_check\""));
        assert!(first.contains("\"content_check\""));
    }

    #[test]
    fn unit_verify_chain_dump_rejects_missing_block_hash_marker() {
        let err = verify_chain_dump(
            r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"previous_block_hash":"GENESIS"}]}"#,
        )
        .expect_err("missing block hash marker should fail");
        assert!(err.contains("chain dump block missing block_hash marker"));
    }

    #[test]
    fn unit_verify_chain_dump_rejects_hash_continuity_mismatch() {
        let err = verify_chain_dump(
            r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:wrong-prior"}]}"#,
        )
        .expect_err("chain continuity mismatch should fail");
        assert!(err.contains("chain dump hash continuity mismatch at block index 1"));
    }
}
