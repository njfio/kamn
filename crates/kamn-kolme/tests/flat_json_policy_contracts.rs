use kamn_kolme::{
    parse_flat_json_value_fields, required_json_string_field, required_positive_u64_json_field,
    KolmeFlatJsonPolicyError, KolmeFlatJsonValue,
};

#[test]
fn functional_parse_flat_json_value_fields_accepts_mixed_scalar_values() {
    let fields = parse_flat_json_value_fields(
        "{\"provider\":\"kolme\",\"height\":72,\"ok\":true,\"meta\":null}",
    )
    .expect("flat json object should parse");
    assert_eq!(
        fields.get("provider"),
        Some(&KolmeFlatJsonValue::String("kolme".to_owned()))
    );
    assert_eq!(
        fields.get("height"),
        Some(&KolmeFlatJsonValue::Number("72".to_owned()))
    );
    assert_eq!(fields.get("ok"), Some(&KolmeFlatJsonValue::Boolean(true)));
    assert_eq!(fields.get("meta"), Some(&KolmeFlatJsonValue::Null));
}

#[test]
fn functional_required_json_string_field_trims_whitespace() {
    let fields = parse_flat_json_value_fields("{\"provider\":\"  kolme-fork  \"}")
        .expect("parse should work");
    let provider = required_json_string_field(&fields, "provider")
        .expect("string field extraction should work");
    assert_eq!(provider, "kolme-fork");
}

#[test]
fn functional_required_positive_u64_json_field_accepts_string_and_number_tokens() {
    let number_fields = parse_flat_json_value_fields("{\"height\":72}").expect("parse should work");
    assert_eq!(
        required_positive_u64_json_field(&number_fields, "height")
            .expect("numeric token should parse"),
        72
    );

    let string_fields =
        parse_flat_json_value_fields("{\"height\":\"73\"}").expect("parse should work");
    assert_eq!(
        required_positive_u64_json_field(&string_fields, "height")
            .expect("string token should parse"),
        73
    );
}

#[test]
fn regression_issue_1747_parse_flat_json_value_fields_rejects_invalid_value_token() {
    // Regression: #1747
    let error = parse_flat_json_value_fields("{\"height\":7.2}")
        .expect_err("unsupported number formats must fail");
    assert_eq!(
        error,
        KolmeFlatJsonPolicyError::MalformedResponse {
            reason: "invalid json value token".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1747_required_positive_u64_json_field_rejects_zero() {
    // Regression: #1747
    let fields = parse_flat_json_value_fields("{\"height\":0}").expect("parse should work");
    let error = required_positive_u64_json_field(&fields, "height")
        .expect_err("zero values must fail closed");
    assert_eq!(
        error,
        KolmeFlatJsonPolicyError::MalformedResponse {
            reason: "height must be positive".to_owned(),
        }
    );
}
