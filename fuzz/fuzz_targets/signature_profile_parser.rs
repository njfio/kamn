#![no_main]

use kamn_core::parse_signature_profile_metadata;
use libfuzzer_sys::fuzz_target;

fn bounded_utf8(data: &[u8], max_len: usize) -> String {
    let mut value = String::from_utf8_lossy(data).to_string();
    if value.len() > max_len {
        value.truncate(max_len);
    }
    value
}

fuzz_target!(|data: &[u8]| {
    let input = bounded_utf8(data, 512);
    let _ = parse_signature_profile_metadata(&input);
});
