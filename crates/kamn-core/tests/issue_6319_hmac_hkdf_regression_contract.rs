use std::fs;
use std::path::Path;

const FORBIDDEN_SIGNATURES: &[&str] = &["fn hkdf_sha256_derive_32(", "fn hmac_sha256("];
const REQUIRED_BACKEND_MARKERS: &[&str] = &["rustcrypto.hkdf.sha256.v1", "rustcrypto.hmac.sha256.v1"];

const GUARDED_PRODUCTION_SOURCES: &[&str] = &[
    // Intentional RED mapping: stale path should fail until GREEN updates guarded source map.
    "../../kamn-core/src/direct_message_crypto.rs",
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

#[test]
fn ac1_forbids_manual_hmac_hkdf_helpers() {
    for relative_path in GUARDED_PRODUCTION_SOURCES {
        let source = load_guarded_source(relative_path);
        for signature in FORBIDDEN_SIGNATURES {
            assert!(
                !source.contains(signature),
                "forbidden helper signature `{signature}` found in {relative_path}"
            );
        }
    }
}

#[test]
fn ac2_requires_rustcrypto_backend_markers() {
    for relative_path in GUARDED_PRODUCTION_SOURCES {
        let source = load_guarded_source(relative_path);
        for marker in REQUIRED_BACKEND_MARKERS {
            assert!(
                source.contains(marker),
                "required backend marker `{marker}` missing in {relative_path}"
            );
        }
    }
}
