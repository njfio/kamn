use crate::verify::support::{
    collect_evidence_json_artifacts, extract_json_string_marker, is_rfc3339_utc_z_timestamp,
    is_sha256_value, strip_json_whitespace,
};
use std::path::Path;

fn extract_anchor_fragment(artifact_json: &str) -> Option<String> {
    let normalized = strip_json_whitespace(artifact_json);
    let anchor_marker = "\"kolme_anchor\":{";
    let anchor_start = normalized.find(anchor_marker)? + anchor_marker.len();
    let anchor_relative_end = normalized[anchor_start..].find('}')?;
    Some(normalized[anchor_start..anchor_start + anchor_relative_end].to_owned())
}

fn extract_anchor_string(artifact_json: &str, marker: &str) -> Option<String> {
    let anchor_fragment = extract_anchor_fragment(artifact_json)?;
    extract_json_string_marker(anchor_fragment.as_str(), marker)
}

fn extract_anchor_block_height(artifact_json: &str) -> Option<u64> {
    let anchor_fragment = extract_anchor_fragment(artifact_json)?;
    let marker = "\"block_height\":";
    let value_start = anchor_fragment.find(marker)? + marker.len();
    let value_fragment = &anchor_fragment[value_start..];
    let digits_len = value_fragment
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    if digits_len == 0 {
        return None;
    }
    value_fragment[..digits_len].parse::<u64>().ok()
}

fn ensure_required_markers(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
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
                "evidence artifact missing {label}: {}",
                artifact_path.display()
            ));
        }
    }
    Ok(())
}

fn extract_required_value(value: Option<String>, error: String) -> Result<String, String> {
    value.filter(|candidate| !candidate.is_empty()).ok_or(error)
}

fn missing_field(path: &Path, field: &str) -> String {
    format!("evidence artifact missing {field}: {}", path.display())
}

fn invalid_field(path: &Path, field: &str) -> String {
    format!(
        "evidence artifact invalid {field} format: {}",
        path.display()
    )
}

fn validate_evidence_hash(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
    let normalized = strip_json_whitespace(artifact_json);
    let evidence_hash = extract_required_value(
        extract_json_string_marker(&normalized, "\"evidence_hash\":\""),
        missing_field(artifact_path, "_verification.evidence_hash"),
    )?;
    if !is_sha256_value(&evidence_hash) {
        return Err(invalid_field(artifact_path, "_verification.evidence_hash"));
    }
    Ok(())
}

fn validate_captured_at(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
    let normalized = strip_json_whitespace(artifact_json);
    let captured_at = extract_required_value(
        extract_json_string_marker(&normalized, "\"captured_at\":\""),
        missing_field(artifact_path, "_verification.captured_at"),
    )?;
    if !is_rfc3339_utc_z_timestamp(&captured_at) {
        return Err(invalid_field(artifact_path, "_verification.captured_at"));
    }
    Ok(())
}

fn validate_anchor_tx_hash(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
    let tx_hash = extract_required_value(
        extract_anchor_string(artifact_json, "\"tx_hash\":\""),
        missing_field(artifact_path, "_verification.kolme_anchor.tx_hash"),
    )?;
    if !is_sha256_value(&tx_hash) {
        return Err(invalid_field(
            artifact_path,
            "_verification.kolme_anchor.tx_hash",
        ));
    }
    Ok(())
}

fn validate_anchor_block_height(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
    if extract_anchor_block_height(artifact_json).is_none() {
        return Err(invalid_field(
            artifact_path,
            "_verification.kolme_anchor.block_height",
        ));
    }
    Ok(())
}

fn validate_anchor_finality(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
    let finality = extract_required_value(
        extract_anchor_string(artifact_json, "\"finality\":\""),
        missing_field(artifact_path, "_verification.kolme_anchor.finality"),
    )?;
    if finality != "FINAL" {
        return Err(format!(
            "evidence artifact invalid _verification.kolme_anchor.finality value: {}",
            artifact_path.display()
        ));
    }
    Ok(())
}

fn validate_evidence_fields(artifact_json: &str, artifact_path: &Path) -> Result<(), String> {
    validate_evidence_hash(artifact_json, artifact_path)?;
    validate_captured_at(artifact_json, artifact_path)?;
    validate_anchor_tx_hash(artifact_json, artifact_path)?;
    validate_anchor_block_height(artifact_json, artifact_path)?;
    validate_anchor_finality(artifact_json, artifact_path)
}

fn validate_artifact(artifact_path: &Path) -> Result<(), String> {
    let artifact_json = std::fs::read_to_string(artifact_path).map_err(|error| {
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
    ensure_required_markers(&artifact_json, artifact_path)?;
    validate_evidence_fields(&artifact_json, artifact_path)
}

/// Verifies PRD section 8.3 `_verification` marker contract on evidence artifacts.
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
        validate_artifact(artifact_path.as_path())?;
    }
    Ok(())
}
