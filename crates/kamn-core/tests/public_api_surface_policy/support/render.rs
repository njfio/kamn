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
    fs::write(&output_path, report).unwrap_or_else(|error| {
        fail(
            "report_output_write_failed",
            &format!("{}: {}", output_path.display(), error),
        )
    });
}

fn header_lines(
    thresholds: &PolicyThresholds,
    status: &PolicyStatus,
    reason_codes: &str,
    report: &ApiSurfaceReport,
) -> Vec<String> {
    vec![
        format!("report_schema_version={}", REPORT_SCHEMA_VERSION),
        format!("policy_schema_version={}", THRESHOLD_SCHEMA_VERSION),
        format!("reason_taxonomy_version={}", REASON_TAXONOMY_VERSION),
        format!("reason_codes={}", reason_codes),
        format!("policy_status={}", status.as_marker()),
        format!("warn_total_delta_max={}", thresholds.warn_total_delta_max),
        format!("fail_total_delta_max={}", thresholds.fail_total_delta_max),
        format!("total_public_items={}", report.total_public_items),
        format!(
            "baseline_total_public_items={}",
            report.baseline_total_public_items
        ),
        format!("public_items_delta={}", report.public_items_delta),
        format!("module_count={}", report.modules.len()),
    ]
}

fn append_module_lines(lines: &mut Vec<String>, modules: &[crate::support::models::ModuleSurface]) {
    for module in modules {
        lines.push(format!(
            "module_public_items.{}={}",
            module.module, module.public_items
        ));
        lines.push(format!(
            "module_public_items_baseline.{}={}",
            module.module, module.baseline_public_items
        ));
        lines.push(format!(
            "module_public_items_delta.{}={}",
            module.module, module.delta_public_items
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
        fs::create_dir_all(parent).unwrap_or_else(|error| {
            fail(
                "report_output_write_failed",
                &format!("failed to create {}: {}", parent.display(), error),
            )
        });
    }
}
