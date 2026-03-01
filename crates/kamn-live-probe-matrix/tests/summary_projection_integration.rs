use kamn_live_probe_matrix::summary_projection::{
    project_live_probe_matrix_summary, LiveProbeMatrixSummary,
};
use kamn_live_probe_matrix::{
    LiveProbeMatrixEntry, LiveProbeMatrixMode, LiveProbeMatrixReport, LiveProbeMatrixStatus,
};

#[test]
fn integration_summary_projection_reports_all_pass_counts_and_status() {
    let report = LiveProbeMatrixReport::new(vec![
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::CliScripted,
            "S-02",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
    ])
    .expect("report");

    let summary = project_live_probe_matrix_summary(&report);
    assert_eq!(
        summary,
        LiveProbeMatrixSummary {
            total_entries: 2,
            pass_entries: 2,
            fail_entries: 0,
            skip_entries: 0,
            overall_status: Some(LiveProbeMatrixStatus::Pass),
        }
    );
}

#[test]
fn integration_summary_projection_reports_mixed_pass_skip_as_fail_closed() {
    let report = LiveProbeMatrixReport::new(vec![
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-02",
            LiveProbeMatrixStatus::Skip,
        )
        .expect("entry"),
    ])
    .expect("report");

    let summary = project_live_probe_matrix_summary(&report);
    assert_eq!(summary.total_entries, 2);
    assert_eq!(summary.pass_entries, 1);
    assert_eq!(summary.fail_entries, 0);
    assert_eq!(summary.skip_entries, 1);
    assert_eq!(summary.overall_status, Some(LiveProbeMatrixStatus::Fail));
}
