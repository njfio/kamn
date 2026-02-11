use kamn_kolme::escape_json_string;

#[test]
fn unit_json_escape_policy_escapes_control_and_quote_characters() {
    let escaped = escape_json_string("a\"b\\c\nd\re\tf");
    assert_eq!(escaped, "a\\\"b\\\\c\\nd\\re\\tf");
}

#[test]
fn functional_json_escape_policy_preserves_plain_ascii() {
    let escaped = escape_json_string("runtime-commit-123");
    assert_eq!(escaped, "runtime-commit-123");
}

#[test]
fn regression_json_escape_policy_remains_deterministic_for_repeated_input() {
    // Regression: #1781
    let value = "\"quoted\"\\slash\n";
    assert_eq!(escape_json_string(value), escape_json_string(value));
}
