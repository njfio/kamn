use sha2::{Digest, Sha256};

use super::artifact_digest::digest_json_without_field;
use super::verify_support::{extract_string, extract_u64, validate_json_delimiters};

pub(super) fn validate_shape(raw: &str, schema: &str, field_count: usize) -> Result<(), String> {
    validate_json_delimiters(raw).map_err(|_| "malformed live task evidence JSON".to_owned())?;
    require_string(raw, "schema_version", schema)?;
    if raw.matches("\":").count() != field_count {
        return Err("live task evidence field allowlist mismatch".to_owned());
    }
    artifact_digest(raw).map(|_| ())
}

pub(super) fn artifact_digest(raw: &str) -> Result<String, String> {
    let expected = extract_string(raw, "artifact_digest")?;
    let actual = digest_json_without_field(raw, "artifact_digest")?;
    if actual.strip_prefix("sha256:") == Some(expected.as_str()) {
        return Ok(expected);
    }
    Err("live task evidence artifact digest mismatch".to_owned())
}

pub(super) fn receipt_pid(raw: &str, actor: &str) -> Result<u64, String> {
    require_string(raw, "actor", actor)?;
    let pid = extract_u64(raw, "pi_process_id")?;
    if pid == 0 {
        Err("live task receipt Pi process ID must be positive".to_owned())
    } else {
        Ok(pid)
    }
}

pub(super) fn require_task_state(raw: &str, task_id: &str) -> Result<(), String> {
    require_string(raw, "task_id", task_id)?;
    require_string(raw, "state", "accepted")
}

pub(super) fn require_string(raw: &str, field: &str, expected: &str) -> Result<(), String> {
    if extract_string(raw, field)? == expected {
        Ok(())
    } else {
        Err(format!("live task evidence {field} mismatch"))
    }
}

pub(super) fn validate_task_id(value: &str) -> Result<(), String> {
    let valid_chars = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte));
    if !value.is_empty() && value.len() <= 200 && valid_chars {
        Ok(())
    } else {
        Err("live task evidence task ID is invalid".to_owned())
    }
}

pub(super) fn reject_secret_path(path: &str) -> Result<(), String> {
    let lower = path.to_ascii_lowercase();
    let secret_like = [
        ".kamn/devnet",
        "auth.json",
        ".env",
        "keypair",
        "id_rsa",
        "oauth",
    ]
    .iter()
    .any(|marker| lower.contains(marker));
    if secret_like {
        Err("refusing secret-like live task evidence path".to_owned())
    } else {
        Ok(())
    }
}

pub(super) fn sha256_hex(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
