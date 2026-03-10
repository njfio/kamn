use super::super::*;

#[test]
fn regression_runtime_observability_endpoint_rejects_custom_path_without_bind_address() {
    assert_parse_error(
        with_pairs(
            cli_args(),
            &[("--role", "processor"), ("--observability-endpoint-metrics-path", "/runtime/metrics")],
        ),
        ConfigError::MissingArgumentValue("--observability-endpoint-bind"),
    );
}
