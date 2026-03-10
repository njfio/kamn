use crate::support::models::{ApiSurfaceReport, ModuleSurface, PolicyStatus, PolicyThresholds};
use crate::support::modules::{count_public_items, module_source_paths};
use crate::support::paths::{fail, repo_path};
use crate::support::thresholds::load_waiver;
use std::collections::BTreeMap;

pub(crate) fn build_report(
    modules: &[String],
    baseline_total_public_items: usize,
    baseline_module_public_items: &BTreeMap<String, usize>,
) -> ApiSurfaceReport {
    let src_root = repo_path("src");
    let mut total_public_items = 0;
    let modules = modules
        .iter()
        .map(|module| build_module_surface(module, &src_root, baseline_module_public_items, &mut total_public_items))
        .collect();
    ApiSurfaceReport {
        total_public_items,
        baseline_total_public_items,
        public_items_delta: total_public_items as i64 - baseline_total_public_items as i64,
        modules,
    }
}

pub(crate) fn evaluate_policy(
    report: &ApiSurfaceReport,
    thresholds: &PolicyThresholds,
) -> (PolicyStatus, String) {
    if report.public_items_delta <= thresholds.warn_total_delta_max {
        return (PolicyStatus::Within, "none".to_owned());
    }
    if report.public_items_delta <= thresholds.fail_total_delta_max {
        return (PolicyStatus::Warn, "public_api_surface_warn_threshold_exceeded".to_owned());
    }
    evaluate_waiver(report, thresholds)
}

fn build_module_surface(
    module: &str,
    src_root: &std::path::Path,
    baselines: &BTreeMap<String, usize>,
    total: &mut usize,
) -> ModuleSurface {
    let source_paths = module_source_paths(module, src_root);
    if source_paths.is_empty() {
        fail("module_source_missing", &format!("module {} has no source files", module));
    }
    let module_public_items = source_paths.iter().map(|path| count_public_items(path)).sum::<usize>();
    let baseline_public_items = *baselines
        .get(module)
        .unwrap_or_else(|| fail("baseline_module_missing", module));
    *total += module_public_items;
    ModuleSurface {
        module: module.to_owned(),
        public_items: module_public_items,
        baseline_public_items,
        delta_public_items: module_public_items as i64 - baseline_public_items as i64,
    }
}

fn evaluate_waiver(report: &ApiSurfaceReport, thresholds: &PolicyThresholds) -> (PolicyStatus, String) {
    if let Some(waiver_path) = &thresholds.waiver_file {
        if waiver_path.is_file() {
            return evaluate_loaded_waiver(report.public_items_delta, waiver_path);
        }
    }
    fail_unwaived(report.public_items_delta, thresholds.fail_total_delta_max)
}

fn evaluate_loaded_waiver(delta: i64, waiver_path: &std::path::Path) -> (PolicyStatus, String) {
    let waiver = load_waiver(waiver_path);
    if delta <= waiver.max_total_delta {
        return (
            PolicyStatus::ExceptionApplied,
            format!("public_api_surface_fail_threshold_exceeded_waived:{}", waiver.mitigation_issue),
        );
    }
    fail(
        "waiver_cap_exceeded",
        &format!("public_items_delta {} exceeds waiver cap {} ({})", delta, waiver.max_total_delta, waiver_path.display()),
    )
}

fn fail_unwaived(delta: i64, fail_total_delta_max: i64) -> ! {
    fail(
        "public_api_surface_fail_threshold_exceeded_unwaived",
        &format!("public_items_delta {} exceeds fail_total_delta_max {}", delta, fail_total_delta_max),
    )
}
