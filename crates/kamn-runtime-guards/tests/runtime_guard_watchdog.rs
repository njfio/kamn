use kamn_runtime_guards::watchdog::{
    WatchdogAlertKind, WatchdogConfig, WatchdogError, WatchdogNode, WatchdogObservation,
    WatchdogSeverity,
};

fn default_watchdog() -> WatchdogNode {
    WatchdogNode::new(WatchdogConfig::default()).expect("valid watchdog config")
}

fn seed_block(watchdog: &mut WatchdogNode) {
    assert!(watchdog
        .observe(WatchdogObservation::block(
            "block-1", "state-1", "genesis", 3, 5,
        ))
        .expect("valid seed block")
        .is_empty());
}

fn anomalous_block_alerts(
    watchdog: &mut WatchdogNode,
) -> Vec<kamn_runtime_guards::watchdog::WatchdogAlert> {
    watchdog
        .observe(WatchdogObservation::block(
            "block-2",
            "state-2",
            "wrong-parent",
            1,
            5,
        ))
        .expect("valid anomalous block")
}

#[test]
fn integration_runtime_guard_watchdog_mixed_sequence_emits_expected_alerts() {
    let mut watchdog = default_watchdog();
    seed_block(&mut watchdog);

    let block_alerts = anomalous_block_alerts(&mut watchdog);
    assert_eq!(block_alerts.len(), 2);
    assert!(block_alerts.iter().any(|alert| {
        alert.severity == WatchdogSeverity::Critical
            && matches!(
                &alert.kind,
                WatchdogAlertKind::InvalidBlockParent {
                    block_id,
                    expected_parent,
                    observed_parent,
                } if block_id == "block-2"
                    && expected_parent == "state-1"
                    && observed_parent == "wrong-parent"
            )
    }));
    assert!(block_alerts.iter().any(|alert| {
        alert.severity == WatchdogSeverity::Critical
            && matches!(
                &alert.kind,
                WatchdogAlertKind::QuorumAnomaly {
                    block_id,
                    observed_signatures,
                    min_required_signatures,
                } if block_id == "block-2"
                    && *observed_signatures == 1
                    && *min_required_signatures == 3
            )
    }));

    let gossip_alerts = watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-1", 10, 2, 10))
        .expect("valid gossip observation");
    assert_eq!(gossip_alerts.len(), 1);
    assert_eq!(gossip_alerts[0].severity, WatchdogSeverity::Warning);
    assert!(matches!(
        &gossip_alerts[0].kind,
        WatchdogAlertKind::CensorshipSignal {
            message_id,
            delivered_recipients,
            expected_recipients,
            observed_ratio_pct,
        } if message_id == "msg-1"
            && *delivered_recipients == 2
            && *expected_recipients == 10
            && *observed_ratio_pct == 20
    ));
}

#[test]
fn integration_runtime_guard_watchdog_single_recipient_gossip_emits_no_alerts() {
    let mut watchdog = default_watchdog();

    let alerts = watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-direct", 1, 1, 1))
        .expect("single-recipient gossip should be valid");

    assert!(alerts.is_empty());
}

#[test]
fn integration_runtime_guard_watchdog_snapshot_tracks_mixed_warning_and_critical_counts() {
    let mut watchdog = default_watchdog();
    seed_block(&mut watchdog);
    anomalous_block_alerts(&mut watchdog);
    watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-1", 10, 2, 10))
        .expect("valid gossip observation");
    watchdog
        .observe(WatchdogObservation::gossip_delivery("msg-direct", 1, 1, 1))
        .expect("valid direct observation");

    let snapshot = watchdog.snapshot();
    assert_eq!(snapshot.total_observations, 4);
    assert_eq!(snapshot.total_alerts, 3);
    assert_eq!(snapshot.warning_alerts, 1);
    assert_eq!(snapshot.critical_alerts, 2);
}

#[test]
fn integration_runtime_guard_watchdog_invalid_config_and_input_fail_closed() {
    assert_eq!(
        WatchdogNode::new(WatchdogConfig {
            min_quorum_signatures: 0,
            min_delivery_ratio_pct: 70,
        }),
        Err(WatchdogError::InvalidConfig(
            "min_quorum_signatures must be greater than zero".to_owned()
        ))
    );

    let mut watchdog = default_watchdog();
    assert_eq!(
        watchdog.observe(WatchdogObservation::block(" ", "state-1", "genesis", 3, 5)),
        Err(WatchdogError::InvalidObservation(
            "block_id must not be empty".to_owned()
        ))
    );
    assert_eq!(
        watchdog.observe(WatchdogObservation::gossip_delivery("msg-1", 0, 0, 0)),
        Err(WatchdogError::InvalidObservation(
            "expected_recipients must be greater than zero".to_owned()
        ))
    );
}
