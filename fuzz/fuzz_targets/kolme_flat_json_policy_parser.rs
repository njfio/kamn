#![no_main]

use kamn_kolme::{
    parse_flat_json_value_fields, parse_provider_key_value_fields, parse_provider_response_fields,
    required_json_string_field, required_positive_u64_json_field,
};
use libfuzzer_sys::fuzz_target;

const DEFAULT_JSON: &str =
    r#"{"txhash":"0xabc123","block_height":21,"ok":true,"note":"alpha,beta:gamma","nullable":null}"#;
const DEFAULT_KV: &str = "txhash=0xabc123\nprovider=kolme\n";

fn bounded_utf8(data: &[u8], max_len: usize) -> String {
    let mut value = String::from_utf8_lossy(data).to_string();
    if value.len() > max_len {
        value.truncate(max_len);
    }
    value
}

fuzz_target!(|data: &[u8]| {
    let input = bounded_utf8(data, 4096);
    let candidate = if input.trim().is_empty() {
        DEFAULT_JSON
    } else {
        input.as_str()
    };

    if let Ok(fields) = parse_flat_json_value_fields(candidate) {
        let _ = required_json_string_field(&fields, "txhash");
        let _ = required_positive_u64_json_field(&fields, "block_height");
    }

    let _ = parse_provider_response_fields(candidate);

    let kv_candidate = if input.trim().is_empty() {
        DEFAULT_KV
    } else {
        input.as_str()
    };
    let _ = parse_provider_key_value_fields(kv_candidate);
});
