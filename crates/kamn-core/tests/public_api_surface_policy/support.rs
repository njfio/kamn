#[path = "support/baseline.rs"]
mod baseline;
#[path = "support/compute.rs"]
mod compute;
#[path = "support/constants.rs"]
pub(crate) mod constants;
#[path = "support/fixtures.rs"]
mod fixtures;
#[path = "support/models.rs"]
pub(crate) mod models;
#[path = "support/modules.rs"]
mod modules;
#[path = "support/paths.rs"]
mod paths;
#[path = "support/policy.rs"]
mod policy;
#[path = "support/render.rs"]
mod render;
#[path = "support/thresholds.rs"]
mod thresholds;

pub(crate) use compute::compute_report_with_policy;
pub(crate) use render::maybe_write_report;
