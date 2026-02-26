fn spawn_delayed_http_health_request(
    bind_addr: &'static str,
    path: &'static str,
    delay_ms: u64,
) -> std::thread::JoinHandle<bool> {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(750);
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
        );

        loop {
            match std::net::TcpStream::connect(bind_addr) {
                Ok(mut stream) => {
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(200)));
                    let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(200)));
                    if std::io::Write::write_all(&mut stream, request.as_bytes()).is_ok() {
                        let mut buffer = [0_u8; 128];
                        let _ = std::io::Read::read(&mut stream, &mut buffer);
                        return true;
                    }
                    return false;
                }
                Err(_) => {
                    if std::time::Instant::now() >= deadline {
                        return false;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    })
}

#[test]
fn regression_runtime_full_supervisor_service_api_lane_early_exit_fails_with_liveness_reason() {
    let request_trigger = spawn_delayed_http_health_request("127.0.0.1:19095", "/healthz", 50);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "250".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "2".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19095".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "500".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err(
        "full supervisor must fail closed when service-api lane exits before daemon completion",
    );
    let request_dispatched = request_trigger
        .join()
        .expect("service-api trigger thread should join");
    assert!(
        request_dispatched,
        "service-api trigger request should be dispatched during daemon execution"
    );
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(ref message) if message.contains("full_supervisor_service_api_lane_liveness_failed")),
        "service-api lane early-exit path must emit deterministic liveness reason code: {error:?}"
    );
}

#[test]
fn regression_runtime_full_supervisor_observability_lane_early_exit_fails_with_liveness_reason() {
    let request_trigger = spawn_delayed_http_health_request("127.0.0.1:19096", "/healthz", 50);
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "full".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "250".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "2".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:19097".to_owned(),
        "--api-idle-timeout-ms".to_owned(),
        "500".to_owned(),
        "--observability-endpoint-bind".to_owned(),
        "127.0.0.1:19096".to_owned(),
        "--observability-endpoint-idle-timeout-ms".to_owned(),
        "500".to_owned(),
    ])
    .expect("full args should parse");

    let error = execute(parsed).expect_err(
        "full supervisor must fail closed when observability lane exits before daemon completion",
    );
    let request_dispatched = request_trigger
        .join()
        .expect("observability trigger thread should join");
    assert!(
        request_dispatched,
        "observability trigger request should be dispatched during daemon execution"
    );
    assert!(
        matches!(error, ConfigError::RuntimeDaemonLifecycle(ref message) if message.contains("full_supervisor_observability_lane_liveness_failed")),
        "observability lane early-exit path must emit deterministic liveness reason code: {error:?}"
    );
}
