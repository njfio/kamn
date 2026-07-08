use std::path::{Path, PathBuf};

use super::report::DemoReportInput;
use super::report_markdown::render_report_markdown;

pub(crate) fn write_reports(
    output_root: &Path,
    run_id: &str,
    report_json: &str,
    input: &DemoReportInput<'_>,
) -> Result<(), String> {
    let report_md = render_report_markdown(input)?;
    write_report_pair(output_root.join(run_id), report_json, report_md.as_str())?;
    refresh_latest(output_root, run_id, report_json, report_md.as_str())
}

fn write_report_pair(root: PathBuf, report_json: &str, report_md: &str) -> Result<(), String> {
    let proof_dir = root.join("proof");
    create_dir(&proof_dir)?;
    write_file(proof_dir.join("report.json"), report_json)?;
    write_file(proof_dir.join("report.md"), report_md)
}

fn refresh_latest(
    output_root: &Path,
    run_id: &str,
    report_json: &str,
    report_md: &str,
) -> Result<(), String> {
    let latest = output_root.join("latest");
    remove_latest(&latest)?;
    write_report_pair(latest.clone(), report_json, report_md)?;
    write_file(latest.join("RUN_ID"), run_id)
}

fn remove_latest(latest: &Path) -> Result<(), String> {
    if latest.exists() {
        std::fs::remove_dir_all(latest)
            .map_err(|error| format!("failed to remove previous latest demo: {error}"))?;
    }
    Ok(())
}

fn create_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|error| {
        format!(
            "failed to create MVP demo directory {}: {error}",
            path.display()
        )
    })
}

fn write_file(path: PathBuf, content: &str) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| {
        format!(
            "failed to write MVP demo artifact {}: {error}",
            path.display()
        )
    })
}
