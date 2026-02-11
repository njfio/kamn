use kamn_kolme::{
    parse_provider_key_value_fields, parse_provider_response_fields,
    KolmeProviderResponsePolicyError,
};

#[test]
fn functional_parse_provider_response_fields_accepts_key_value_payload() {
    let fields = parse_provider_response_fields("status=submitted\nprovider=kolme\n")
        .expect("key/value payload should parse");
    assert_eq!(fields.get("status"), Some(&"submitted".to_owned()));
    assert_eq!(fields.get("provider"), Some(&"kolme".to_owned()));
}

#[test]
fn functional_parse_provider_response_fields_accepts_flat_json_object() {
    let fields = parse_provider_response_fields(
        "{\"status\":\"duplicate\",\"provider\":\"kolme\",\"tx_hash\":\"ab12cd34\"}",
    )
    .expect("flat json payload should parse");
    assert_eq!(fields.get("status"), Some(&"duplicate".to_owned()));
    assert_eq!(fields.get("provider"), Some(&"kolme".to_owned()));
    assert_eq!(fields.get("tx_hash"), Some(&"ab12cd34".to_owned()));
}

#[test]
fn regression_issue_1745_parse_provider_response_fields_rejects_invalid_key_value_line() {
    // Regression: #1745
    let error = parse_provider_response_fields("status\nprovider=kolme")
        .expect_err("invalid key/value line must fail");
    assert_eq!(
        error,
        KolmeProviderResponsePolicyError::MalformedResponse {
            reason: "invalid key/value response line: status".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1745_parse_provider_response_fields_rejects_unterminated_json_entry() {
    // Regression: #1745
    let error = parse_provider_response_fields("{\"status\":\"submitted}")
        .expect_err("unterminated json strings must fail");
    assert_eq!(
        error,
        KolmeProviderResponsePolicyError::MalformedResponse {
            reason: "invalid json response: unterminated quoted string".to_owned(),
        }
    );
}

#[test]
fn unit_parse_provider_key_value_fields_rejects_empty_body() {
    let error = parse_provider_key_value_fields("\n \n")
        .expect_err("empty key/value payload must fail closed");
    assert_eq!(
        error,
        KolmeProviderResponsePolicyError::MalformedResponse {
            reason: "response body must contain at least one field".to_owned(),
        }
    );
}
