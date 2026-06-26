use crate::support::baseline::load_baseline;
use crate::support::models::{ApiSurfaceReport, PolicyStatus, PolicyThresholds};
use crate::support::modules::parse_public_modules;
use crate::support::paths::{read_file, repo_path};
use crate::support::policy::{build_report, evaluate_policy};
use crate::support::render::render_report;
use crate::support::thresholds::load_thresholds;

pub(crate) fn compute_report_with_policy() -> (
    ApiSurfaceReport,
    PolicyThresholds,
    PolicyStatus,
    String,
    String,
) {
    let modules = parse_public_modules(&read_file(
        &repo_path("src/lib.rs"),
        "module_source_missing",
    ));
    let baseline_path = repo_path("../../fixtures/ci/kamn_core_public_api_surface_baseline.env");
    let threshold_path = repo_path("../../.ci/kamn-core-public-api-surface-thresholds.env");
    let (baseline_total_public_items, baseline_module_public_items) =
        load_baseline(&baseline_path, &modules);
    let thresholds = load_thresholds(&threshold_path);
    let report = build_report(
        &modules,
        baseline_total_public_items,
        &baseline_module_public_items,
    );
    let (status, reason_codes) = evaluate_policy(&report, &thresholds);
    let rendered = render_report(&report, &thresholds, &status, &reason_codes);
    (report, thresholds, status, reason_codes, rendered)
}
