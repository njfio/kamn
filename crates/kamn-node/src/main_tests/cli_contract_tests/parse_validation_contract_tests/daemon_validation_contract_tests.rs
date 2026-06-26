use super::super::*;

#[test]
fn rejects_invalid_daemon_control_argument() {
    assert_parse_error(
        with_pairs(
            daemon_args(),
            &[
                ("--daemon-max-ticks", "abc"),
                ("--daemon-tick-interval-ms", "25"),
            ],
        ),
        ConfigError::InvalidDaemonControlArgument("abc".to_owned()),
    );
}

#[test]
fn rejects_invalid_daemon_lifecycle_event_argument() {
    assert_parse_error(
        with_pairs(
            daemon_args(),
            &[
                ("--daemon-max-ticks", "3"),
                ("--daemon-tick-interval-ms", "25"),
                ("--daemon-peer-id", "peer-alpha"),
                ("--daemon-lifecycle-event", "resume"),
            ],
        ),
        ConfigError::InvalidDaemonLifecycleEvent("resume".to_owned()),
    );
}
