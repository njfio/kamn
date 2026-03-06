use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 5] = [
    "fn integration_top_level_and_module_key_binding_generation_match()",
    "fn integration_generated_agent_did_exposes_and_verifies_key_binding_fingerprint()",
    "fn integration_invalid_public_key_hex_surfaces_shared_boundary_error()",
    "fn integration_canonical_parse_helpers_preserve_missing_id_and_invalid_shape_errors()",
    "fn integration_shared_did_boundary_types_are_constructible_via_kamn_types()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_key_binding_boundary_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-types/tests/key_binding_boundary_integration.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "kamn-types key-binding boundary target should contain marker: {marker}"
        );
    }
}
