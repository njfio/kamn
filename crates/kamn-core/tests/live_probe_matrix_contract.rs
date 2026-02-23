use kamn_core::{
    LiveProbeMatrixEntry, LiveProbeMatrixError, LiveProbeMatrixMode, LiveProbeMatrixReport,
    LiveProbeMatrixStatus,
};

#[test]
fn spec_c01_live_probe_matrix_module_exports_are_constructible() {
    let report = LiveProbeMatrixReport::new(vec![
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::CliScripted,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::McpTau,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
    ])
    .expect("report");

    assert_eq!(report.entries().len(), 3);
    assert_eq!(
        report.status_for(LiveProbeMatrixMode::SdkDirect, "S-01"),
        Some(LiveProbeMatrixStatus::Pass)
    );
}

#[test]
fn spec_c02_live_probe_matrix_fails_closed_on_empty_scenario_id() {
    let error = LiveProbeMatrixEntry::new(
        LiveProbeMatrixMode::SdkDirect,
        "",
        LiveProbeMatrixStatus::Pass,
    )
    .expect_err("empty scenario id must fail closed");

    assert_eq!(error, LiveProbeMatrixError::EmptyScenarioId);
}

#[test]
fn spec_c03_live_probe_matrix_fails_closed_on_duplicate_mode_scenario_pair() {
    let duplicate = vec![
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::McpTau,
            "S-04",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::McpTau,
            "S-04",
            LiveProbeMatrixStatus::Fail,
        )
        .expect("entry"),
    ];

    let error = LiveProbeMatrixReport::new(duplicate).expect_err("duplicate row should fail");
    assert!(matches!(
        error,
        LiveProbeMatrixError::DuplicateModeScenario {
            mode: LiveProbeMatrixMode::McpTau,
            ..
        }
    ));
}

#[test]
fn spec_c04_live_probe_matrix_reports_fail_closed_aggregate_for_mixed_pass_skip() {
    let report = LiveProbeMatrixReport::new(vec![
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-04",
            LiveProbeMatrixStatus::Skip,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::CliScripted,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::CliScripted,
            "S-04",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
    ])
    .expect("report");

    assert_eq!(
        report.mode_status(LiveProbeMatrixMode::SdkDirect),
        Some(LiveProbeMatrixStatus::Fail)
    );
    assert_eq!(
        report.mode_status(LiveProbeMatrixMode::CliScripted),
        Some(LiveProbeMatrixStatus::Pass)
    );
    assert_eq!(report.overall_status(), Some(LiveProbeMatrixStatus::Fail));
}

#[test]
fn spec_c05_live_probe_matrix_all_pass_matrix_reports_pass() {
    let report = LiveProbeMatrixReport::new(vec![
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::SdkDirect,
            "S-04",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::CliScripted,
            "S-01",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
        LiveProbeMatrixEntry::new(
            LiveProbeMatrixMode::McpTau,
            "S-06",
            LiveProbeMatrixStatus::Pass,
        )
        .expect("entry"),
    ])
    .expect("report");

    assert_eq!(report.overall_status(), Some(LiveProbeMatrixStatus::Pass));
    assert_eq!(
        report
            .mode_status_map()
            .get(&LiveProbeMatrixMode::SdkDirect),
        Some(&LiveProbeMatrixStatus::Pass)
    );
    assert_eq!(
        report.mode_status_map().get(&LiveProbeMatrixMode::McpTau),
        Some(&LiveProbeMatrixStatus::Pass)
    );
}
