use super::super::*;

#[test]
fn rejects_daemon_without_max_ticks() {
    let args = with_pairs(daemon_args(), &[("--daemon-tick-interval-ms", "25")]);
    assert_parse_error(args, missing_arg("--daemon-max-ticks"));
}

#[test]
fn rejects_daemon_without_tick_interval() {
    let args = with_pairs(daemon_args(), &[("--daemon-max-ticks", "3")]);
    assert_parse_error(args, missing_arg("--daemon-tick-interval-ms"));
}

#[test]
fn rejects_full_without_max_ticks() {
    let args = with_pairs(
        full_args(),
        &[
            ("--daemon-tick-interval-ms", "25"),
            ("--api-bind", "127.0.0.1:19083"),
        ],
    );
    assert_parse_error(args, missing_arg("--daemon-max-ticks"));
}

#[test]
fn rejects_full_without_api_bind() {
    let args = with_pairs(
        full_args(),
        &[
            ("--daemon-max-ticks", "3"),
            ("--daemon-tick-interval-ms", "25"),
        ],
    );
    assert_parse_error(args, missing_arg("--api-bind"));
}

#[test]
fn rejects_daemon_shutdown_signal_without_drain_ticks() {
    let args = daemon_shutdown_args();
    assert_parse_error(args, missing_arg("--daemon-shutdown-drain-ticks"));
}

#[test]
fn rejects_daemon_shutdown_signal_without_timeout_ticks() {
    let args = with_pairs(
        daemon_args(),
        &[
            ("--daemon-max-ticks", "8"),
            ("--daemon-tick-interval-ms", "25"),
            ("--daemon-shutdown-signal-tick", "3"),
            ("--daemon-shutdown-drain-ticks", "2"),
        ],
    );
    assert_parse_error(args, missing_arg("--daemon-shutdown-timeout-ticks"));
}

#[test]
fn rejects_daemon_shutdown_controls_without_signal_tick() {
    let args = with_pairs(
        daemon_args(),
        &[
            ("--daemon-max-ticks", "8"),
            ("--daemon-tick-interval-ms", "25"),
            ("--daemon-shutdown-drain-ticks", "2"),
            ("--daemon-shutdown-timeout-ticks", "4"),
        ],
    );
    assert_parse_error(args, missing_arg("--daemon-shutdown-signal-tick"));
}

fn daemon_shutdown_args() -> Vec<String> {
    with_pairs(
        daemon_args(),
        &[
            ("--daemon-max-ticks", "8"),
            ("--daemon-tick-interval-ms", "25"),
            ("--daemon-shutdown-signal-tick", "3"),
            ("--daemon-shutdown-timeout-ticks", "2"),
        ],
    )
}
