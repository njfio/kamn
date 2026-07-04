use crate::support::constants::{
    REASON_TAXONOMY_VERSION, REPORT_SCHEMA_VERSION, THRESHOLD_SCHEMA_VERSION,
};
use crate::support::models::{ApiSurfaceReport, PolicyStatus, PolicyThresholds};
use crate::support::paths::fail;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn render_report(
    report: &ApiSurfaceReport,
    thresholds: &PolicyThresholds,
    status: &PolicyStatus,
    reason_codes: &str,
) -> String {
    let mut lines = header_lines(thresholds, status, reason_codes, report);
    append_module_lines(&mut lines, &report.modules);
    lines.join("\n") + "\n"
}

pub(crate) fn maybe_write_report(report: &str) {
    let Some(output_path) = output_path() else {
        return;
    };
    create_parent_dir(&output_path);
    let output_path_display = output_path.display();
    fs::write(&output_path, report).unwrap_or_else(|error| {
        fail(
            "report_output_write_failed",
            &format!("{output_path_display}: {error}"),
        )
    });
}

fn header_lines(
    thresholds: &PolicyThresholds,
    status: &PolicyStatus,
    reason_codes: &str,
    report: &ApiSurfaceReport,
) -> Vec<String> {
    let mut lines = schema_header_lines(reason_codes);
    lines.extend(threshold_header_lines(thresholds, status));
    lines.extend(report_count_lines(report));
    lines
}

fn schema_header_lines(reason_codes: &str) -> Vec<String> {
    vec![
        format!("report_schema_version={REPORT_SCHEMA_VERSION}"),
        format!("policy_schema_version={THRESHOLD_SCHEMA_VERSION}"),
        format!("reason_taxonomy_version={REASON_TAXONOMY_VERSION}"),
        format!("reason_codes={reason_codes}"),
    ]
}

fn threshold_header_lines(thresholds: &PolicyThresholds, status: &PolicyStatus) -> Vec<String> {
    let policy_status = status.as_marker();
    let warn_total_delta_max = thresholds.warn_total_delta_max;
    let fail_total_delta_max = thresholds.fail_total_delta_max;
    vec![
        format!("policy_status={policy_status}"),
        format!("warn_total_delta_max={warn_total_delta_max}"),
        format!("fail_total_delta_max={fail_total_delta_max}"),
    ]
}

fn report_count_lines(report: &ApiSurfaceReport) -> Vec<String> {
    let total_public_items = report.total_public_items;
    let baseline_total_public_items = report.baseline_total_public_items;
    let public_items_delta = report.public_items_delta;
    let module_count = report.modules.len();
    vec![
        format!("total_public_items={total_public_items}"),
        format!("baseline_total_public_items={baseline_total_public_items}"),
        format!("public_items_delta={public_items_delta}"),
        format!("module_count={module_count}"),
    ]
}

fn append_module_lines(lines: &mut Vec<String>, modules: &[crate::support::models::ModuleSurface]) {
    for module in modules {
        let module_name = &module.module;
        let public_items = module.public_items;
        let baseline_public_items = module.baseline_public_items;
        let delta_public_items = module.delta_public_items;
        lines.push(format!("module_public_items.{module_name}={public_items}"));
        lines.push(format!(
            "module_public_items_baseline.{module_name}={baseline_public_items}"
        ));
        lines.push(format!(
            "module_public_items_delta.{module_name}={delta_public_items}"
        ));
    }
}

fn output_path() -> Option<PathBuf> {
    std::env::var("KAMN_CORE_PUBLIC_API_SURFACE_REPORT_OUTPUT")
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
}

fn create_parent_dir(output_path: &Path) {
    if let Some(parent) = output_path.parent() {
        let parent_display = parent.display();
        fs::create_dir_all(parent).unwrap_or_else(|error| {
            fail(
                "report_output_write_failed",
                &format!("failed to create {parent_display}: {error}"),
            )
        });
    }
}
