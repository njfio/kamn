use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModuleSurface {
    pub(crate) module: String,
    pub(crate) public_items: usize,
    pub(crate) baseline_public_items: usize,
    pub(crate) delta_public_items: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApiSurfaceReport {
    pub(crate) total_public_items: usize,
    pub(crate) baseline_total_public_items: usize,
    pub(crate) public_items_delta: i64,
    pub(crate) modules: Vec<ModuleSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PolicyThresholds {
    pub(crate) warn_total_delta_max: i64,
    pub(crate) fail_total_delta_max: i64,
    pub(crate) waiver_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PolicyWaiver {
    pub(crate) mitigation_issue: String,
    pub(crate) max_total_delta: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PolicyStatus {
    Within,
    Warn,
    ExceptionApplied,
}

impl PolicyStatus {
    pub(crate) fn as_marker(&self) -> &'static str {
        match self {
            Self::Within => "within",
            Self::Warn => "warn",
            Self::ExceptionApplied => "exception-applied",
        }
    }
}
