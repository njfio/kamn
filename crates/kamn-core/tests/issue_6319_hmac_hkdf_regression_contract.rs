use std::fs;
use std::path::Path;

const FORBIDDEN_HELPER_NAMES: &[&str] = &["hkdf_sha256_derive_32", "hmac_sha256"];
const REQUIRED_BACKEND_MARKERS: &[&str] = &["rustcrypto.hkdf.sha256.v1", "rustcrypto.hmac.sha256.v1"];

const GUARDED_PRODUCTION_SOURCES: &[&str] = &[
    "../kamn-crypto/src/direct_message_crypto.rs",
    "src/group_channel_crypto.rs",
];

fn load_guarded_source(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(path.as_path()).unwrap_or_else(|error| {
        panic!(
            "guarded source must be readable at {}: {error}",
            path.display()
        )
    })
}

fn source_defines_forbidden_helper(source: &str, helper_name: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        let fn_prefix = format!("fn {helper_name}(");
        let pub_fn_prefix = format!("pub fn {helper_name}(");
        let pub_crate_fn_prefix = format!("pub(crate) fn {helper_name}(");
        trimmed.starts_with(fn_prefix.as_str())
            || trimmed.starts_with(pub_fn_prefix.as_str())
            || trimmed.starts_with(pub_crate_fn_prefix.as_str())
    })
}

fn for_each_guarded_source(mut check: impl FnMut(&str, &str)) {
    for relative_path in GUARDED_PRODUCTION_SOURCES {
        let source = load_guarded_source(relative_path);
        check(relative_path, source.as_str());
    }
}

#[test]
fn ac1_forbids_manual_hmac_hkdf_helpers() {
    for_each_guarded_source(|relative_path, source| {
        for helper_name in FORBIDDEN_HELPER_NAMES {
            assert!(
                !source_defines_forbidden_helper(source, helper_name),
                "forbidden helper definition `fn {helper_name}(` found in {relative_path}"
            );
        }
    });
}

#[test]
fn ac2_requires_rustcrypto_backend_markers() {
    for_each_guarded_source(|relative_path, source| {
        for marker in REQUIRED_BACKEND_MARKERS {
            assert!(
                source.contains(marker),
                "required backend marker `{marker}` missing in {relative_path}"
            );
        }
    });
}
