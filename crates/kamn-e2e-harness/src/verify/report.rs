use crate::verify::manifest::verify_manifest;

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

fn passing_check(detail: &str) -> VerificationCheck {
    VerificationCheck {
        status: "PASS".to_owned(),
        detail: detail.to_owned(),
    }
}

/// Builds a deterministic verification report from an evidence manifest payload.
pub fn generate_verification_report(manifest_json: &str) -> Result<VerificationReport, String> {
    verify_manifest(manifest_json)?;
    Ok(VerificationReport {
        schema_check: passing_check("schema_version validated"),
        proof_check: passing_check("proof markers present"),
        chain_check: passing_check("chain markers present"),
        content_check: passing_check("content markers present"),
    })
}

/// Builds and renders deterministic verification report JSON.
pub fn generate_verification_report_json(manifest_json: &str) -> Result<String, String> {
    let report = generate_verification_report(manifest_json)?;
    Ok(report.to_json())
}
