const API_CODEC_SOURCE: &str = include_str!("../src/api_codec.rs");
const BLOCK_SCAN_POLICY_SOURCE: &str = include_str!("../src/block_scan_policy.rs");
const FLAT_JSON_POLICY_SOURCE: &str = include_str!("../src/flat_json_policy.rs");
const NOTIFICATION_POLICY_SOURCE: &str = include_str!("../src/notification_policy.rs");
const PROVIDER_RESPONSE_POLICY_SOURCE: &str = include_str!("../src/provider_response_policy.rs");
const ENDPOINT_POLICY_SOURCE: &str = include_str!("../src/endpoint_policy.rs");

#[test]
fn spec_c03_kolme_json_string_helper_is_not_duplicated_across_modules() {
    // Regression: #5935
    for (module, source) in [
        ("api_codec", API_CODEC_SOURCE),
        ("block_scan_policy", BLOCK_SCAN_POLICY_SOURCE),
        ("flat_json_policy", FLAT_JSON_POLICY_SOURCE),
        ("notification_policy", NOTIFICATION_POLICY_SOURCE),
        ("provider_response_policy", PROVIDER_RESPONSE_POLICY_SOURCE),
    ] {
        assert!(
            !source.contains("fn parse_json_string("),
            "{module} must use canonical parse_json_string helper"
        );
    }
}

#[test]
fn spec_c03_kolme_percent_encode_helper_is_not_duplicated_across_modules() {
    // Regression: #5935
    for (module, source) in [
        ("api_codec", API_CODEC_SOURCE),
        ("endpoint_policy", ENDPOINT_POLICY_SOURCE),
    ] {
        assert!(
            !source.contains("fn percent_encode("),
            "{module} must use canonical percent_encode helper"
        );
    }
}
