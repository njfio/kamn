use std::collections::BTreeMap;

use kamn_live_probe_matrix::summary_projection::project_live_probe_matrix_summary;
use kamn_live_probe_matrix::{
    LiveProbeMatrixEntry, LiveProbeMatrixMode, LiveProbeMatrixReport, LiveProbeMatrixStatus,
};

fn entry(
    mode: LiveProbeMatrixMode,
    scenario_id: &str,
    status: LiveProbeMatrixStatus,
) -> LiveProbeMatrixEntry {
    LiveProbeMatrixEntry::new(mode, scenario_id, status).expect("entry")
}

#[test]
fn integration_empty_report_has_no_overall_status_or_modes() {
    let report = LiveProbeMatrixReport::new(vec![]).expect("empty report should validate");

    assert_eq!(report.overall_status(), None);
    assert_eq!(report.mode_status(LiveProbeMatrixMode::SdkDirect), None);
    assert!(report.mode_status_map().is_empty());
}

#[test]
fn integration_all_skip_report_aggregates_to_skip() {
    let report = LiveProbeMatrixReport::new(vec![
        entry(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Skip,
        ),
        entry(
            LiveProbeMatrixMode::SdkDirect,
            "S-02",
            LiveProbeMatrixStatus::Skip,
        ),
        entry(
            LiveProbeMatrixMode::McpTau,
            "S-03",
            LiveProbeMatrixStatus::Skip,
        ),
    ])
    .expect("report");

    assert_eq!(
        report.mode_status(LiveProbeMatrixMode::SdkDirect),
        Some(LiveProbeMatrixStatus::Skip)
    );
    assert_eq!(
        report.mode_status(LiveProbeMatrixMode::McpTau),
        Some(LiveProbeMatrixStatus::Skip)
    );
    assert_eq!(report.overall_status(), Some(LiveProbeMatrixStatus::Skip));
}

#[test]
fn integration_summary_projection_preserves_all_skip_counts_and_status() {
    let report = LiveProbeMatrixReport::new(vec![
        entry(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Skip,
        ),
        entry(
            LiveProbeMatrixMode::McpTau,
            "S-02",
            LiveProbeMatrixStatus::Skip,
        ),
    ])
    .expect("report");

    let summary = project_live_probe_matrix_summary(&report);
    assert_eq!(summary.total_entries, 2);
    assert_eq!(summary.pass_entries, 0);
    assert_eq!(summary.fail_entries, 0);
    assert_eq!(summary.skip_entries, 2);
    assert_eq!(summary.overall_status, Some(LiveProbeMatrixStatus::Skip));
}

#[test]
fn integration_status_for_trims_lookup_and_rejects_empty_lookup() {
    let report = LiveProbeMatrixReport::new(vec![entry(
        LiveProbeMatrixMode::CliScripted,
        "S-09",
        LiveProbeMatrixStatus::Pass,
    )])
    .expect("report");

    assert_eq!(
        report.status_for(LiveProbeMatrixMode::CliScripted, "  S-09  "),
        Some(LiveProbeMatrixStatus::Pass)
    );
    assert_eq!(report.status_for(LiveProbeMatrixMode::CliScripted, "   "), None);
}

#[test]
fn integration_mode_status_map_is_deterministic_for_mixed_modes() {
    let report = LiveProbeMatrixReport::new(vec![
        entry(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        ),
        entry(
            LiveProbeMatrixMode::SdkDirect,
            "S-02",
            LiveProbeMatrixStatus::Skip,
        ),
        entry(
            LiveProbeMatrixMode::McpTau,
            "S-03",
            LiveProbeMatrixStatus::Skip,
        ),
    ])
    .expect("report");

    let expected = BTreeMap::from([
        (LiveProbeMatrixMode::SdkDirect, LiveProbeMatrixStatus::Fail),
        (LiveProbeMatrixMode::McpTau, LiveProbeMatrixStatus::Skip),
    ]);

    assert_eq!(report.mode_status_map(), expected);
    assert!(
        !report
            .mode_status_map()
            .contains_key(&LiveProbeMatrixMode::CliScripted)
    );
}
