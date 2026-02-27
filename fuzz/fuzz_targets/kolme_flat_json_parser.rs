#![no_main]

use kamn_kolme::{parse_flat_json_value_fields, parse_provider_response_fields};
use libfuzzer_sys::fuzz_target;

const DEFAULT_JSON: &str = r#"{"provider":"kolme-fork","height":1,"confirmed":true}"#;
const DEFAULT_KV: &str = "provider=kolme-fork\nstatus=ok\n";

fn bounded_utf8(data: &[u8], max_len: usize) -> String {
    let mut value = String::from_utf8_lossy(data).to_string();
    if value.len() > max_len {
        value.truncate(max_len);
    }
    value
}

fuzz_target!(|data: &[u8]| {
    let input = bounded_utf8(data, 4096);
    let trimmed = input.trim();

    let json_candidate = if trimmed.starts_with('{') {
        trimmed
    } else {
        DEFAULT_JSON
    };
    let kv_candidate = if trimmed.contains('=') {
        trimmed
    } else {
        DEFAULT_KV
    };

    let _ = parse_flat_json_value_fields(json_candidate);
    let _ = parse_provider_response_fields(json_candidate);
    let _ = parse_provider_response_fields(kv_candidate);
});
