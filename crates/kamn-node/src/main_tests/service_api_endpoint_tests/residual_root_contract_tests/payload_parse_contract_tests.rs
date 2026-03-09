use super::*;

#[test]
fn regression_service_api_payload_parse_reason_codes_fail_closed() {
    let _env = acquire_service_api_test_env();
    let syntax_error = parse_service_api_payload::<ServiceApiHealthBody>("{\"status\":\"ok\"");
    let syntax_reason = syntax_error.expect_err("invalid json syntax should fail closed");
    assert!(
        syntax_reason.starts_with("service_api_payload_json_syntax_invalid:"),
        "unexpected syntax reason marker: {syntax_reason}"
    );

    let structure_error = parse_service_api_payload::<ServiceApiHealthBody>(
        "{\"status\":\"ok\",\"runtime_mode\":\"api\"}",
    );
    let structure_reason =
        structure_error.expect_err("invalid payload structure should fail closed");
    assert!(
        structure_reason.starts_with("service_api_payload_structure_invalid:"),
        "unexpected structure reason marker: {structure_reason}"
    );
}
