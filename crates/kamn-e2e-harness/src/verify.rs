#[path = "verify/chain_dump.rs"]
mod chain_dump;
#[path = "verify/evidence.rs"]
mod evidence;
#[path = "verify/manifest.rs"]
mod manifest;
#[path = "verify/report.rs"]
mod report;
#[path = "verify/support.rs"]
mod support;

pub use chain_dump::verify_chain_dump;
pub use evidence::validate_evidence_verification_blocks;
pub use manifest::verify_manifest;
pub use report::{
    generate_verification_report, generate_verification_report_json, VerificationCheck,
    VerificationReport,
};

#[cfg(test)]
#[path = "verify/tests.rs"]
mod tests;
