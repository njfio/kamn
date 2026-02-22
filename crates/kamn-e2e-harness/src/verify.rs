use crate::evidence::MANIFEST_SCHEMA_VERSION;

/// Verifies a minimal JSON manifest payload using deterministic marker checks.
pub fn verify_manifest(manifest_json: &str) -> Result<(), String> {
    if !manifest_json.contains(MANIFEST_SCHEMA_VERSION) {
        return Err("manifest schema version mismatch".to_owned());
    }
    if !manifest_json.contains("\"execution_mode\":") {
        return Err("manifest missing execution_mode".to_owned());
    }
    if !manifest_json.contains("\"scenarios\":") {
        return Err("manifest missing scenarios".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify_manifest;

    #[test]
    fn unit_verify_manifest_rejects_missing_schema_marker() {
        let result = verify_manifest(r#"{"execution_mode":"sdk-direct","scenarios":[]}"#);
        assert!(result.is_err());
    }
}
