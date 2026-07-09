use std::path::PathBuf;

use super::verify_support::extract_string;

pub(crate) struct LocalArtifactPaths {
    pub(crate) state_dir: PathBuf,
    pub(crate) audit_export: PathBuf,
    pub(crate) localhost_artifact: PathBuf,
    pub(crate) localhost_output: PathBuf,
    pub(crate) vertical_log: PathBuf,
    pub(crate) websocket_log: PathBuf,
    pub(crate) devnet_log: PathBuf,
}

impl LocalArtifactPaths {
    pub(crate) fn from_report(report_json: &str) -> Result<Self, String> {
        Ok(Self {
            state_dir: artifact_path(report_json, "state_dir")?,
            audit_export: artifact_path(report_json, "audit_export")?,
            localhost_artifact: artifact_path(report_json, "localhost_signed_demo_artifact")?,
            localhost_output: artifact_path(report_json, "localhost_signed_demo_output")?,
            vertical_log: artifact_path(report_json, "service_api_vertical_slice_output")?,
            websocket_log: artifact_path(report_json, "service_api_websocket_output")?,
            devnet_log: artifact_path(report_json, "devnet_settlement_output")?,
        })
    }
}

fn artifact_path(report_json: &str, field: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(extract_string(report_json, field)?))
}
