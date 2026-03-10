use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct Baseline {
    pub(crate) shell_test_file_count: i64,
    pub(crate) rust_test_file_count: i64,
    pub(crate) docs_rust_test_file_count: i64,
    pub(crate) shell_to_rust_ratio: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct Thresholds {
    pub(crate) allowed_shell_test_file_delta_max: i64,
    pub(crate) allowed_ratio_delta_max: f64,
    pub(crate) waiver_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct Waiver {
    pub(crate) mitigation_issue: String,
    pub(crate) max_shell_test_file_delta: i64,
    pub(crate) max_ratio_delta: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct CurrentSurface {
    pub(crate) shell_test_file_count: i64,
    pub(crate) rust_test_file_count: i64,
    pub(crate) docs_rust_test_file_count: i64,
    pub(crate) shell_to_rust_ratio: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct Evaluation {
    pub(crate) policy_status: &'static str,
    pub(crate) final_decision: &'static str,
    pub(crate) reason_codes: Vec<&'static str>,
}
