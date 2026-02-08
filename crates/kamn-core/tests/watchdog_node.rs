use kamn_core::{
    WatchdogAlertKind, WatchdogConfig, WatchdogError, WatchdogNode, WatchdogObservation,
    WatchdogSeverity,
};

#[test]
fn invalid_block_parent_link_triggers_critical_alert() {
    let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("config should build");

    watchdog
        .observe(WatchdogObservation::block(
            "block-1", "state-1", "state-0", 3, 5,
        ))
        .expect("first block should be accepted");

    let alerts = watchdog
        .observe(WatchdogObservation::block(
            "block-2",
            "state-2",
            "unexpected-parent",
            3,
            4,
        ))
        .expect("second block should evaluate");

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, WatchdogSeverity::Critical);
    assert!(matches!(
        alerts[0].kind,
        WatchdogAlertKind::InvalidBlockParent { .. }
    ));
}

#[test]
fn censorship_signal_triggers_alert_when_delivery_below_threshold() {
    let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("config should build");

    let alerts = watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-1", 10, 6, 10))
        .expect("gossip should evaluate");

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, WatchdogSeverity::Warning);
    assert!(matches!(
        alerts[0].kind,
        WatchdogAlertKind::CensorshipSignal { .. }
    ));
}

#[test]
fn quorum_anomaly_triggers_critical_alert() {
    let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("config should build");

    let alerts = watchdog
        .observe(WatchdogObservation::block(
            "block-1", "state-1", "state-0", 2, 5,
        ))
        .expect("block should evaluate");

    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, WatchdogSeverity::Critical);
    assert!(matches!(
        alerts[0].kind,
        WatchdogAlertKind::QuorumAnomaly { .. }
    ));
}

#[test]
fn integration_snapshot_rolls_alert_counts_by_severity() {
    let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("config should build");

    watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-ok", 12, 11, 12))
        .expect("healthy gossip should evaluate");
    watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-warn", 12, 7, 12))
        .expect("degraded gossip should evaluate");
    watchdog
        .observe(WatchdogObservation::block(
            "block-critical",
            "state-10",
            "state-9",
            1,
            5,
        ))
        .expect("critical block should evaluate");

    let snapshot = watchdog.snapshot();
    assert_eq!(snapshot.total_observations, 3);
    assert_eq!(snapshot.warning_alerts, 1);
    assert_eq!(snapshot.critical_alerts, 1);
    assert_eq!(snapshot.total_alerts, 2);
}

#[test]
fn regression_single_recipient_target_not_flagged_as_censorship() {
    // Regression: #204
    let mut watchdog = WatchdogNode::new(WatchdogConfig::default()).expect("config should build");

    let alerts = watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-direct", 1, 1, 1))
        .expect("single-recipient delivery should evaluate");

    assert!(alerts.is_empty());
}

#[test]
fn rejects_invalid_configuration() {
    assert_eq!(
        WatchdogNode::new(WatchdogConfig {
            min_quorum_signatures: 0,
            min_delivery_ratio_pct: 60,
        }),
        Err(WatchdogError::InvalidConfig(
            "min_quorum_signatures must be greater than zero".to_owned()
        ))
    );
}
