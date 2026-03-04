use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn cli_module_extraction_contract_declares_config_layering_module() {
    let cli_rs = read_repo_file("src/cli.rs");
    assert!(
        cli_rs.contains("mod cli_config_layering;"),
        "cli.rs should declare cli_config_layering module"
    );
}

#[test]
fn cli_module_extraction_contract_removes_inline_config_layering_helpers() {
    let cli_rs = read_repo_file("src/cli.rs");
    for marker in [
        "fn read_env_var_trimmed(",
        "fn parse_bool_override(",
        "fn push_key_value_flag(",
        "fn map_config_entry_to_args(",
        "fn parse_config_file_args(",
        "fn append_env_override(",
        "fn collect_env_override_args(",
        "fn extract_config_file_path(",
        "fn build_layered_cli_args(",
    ] {
        assert!(
            !cli_rs.contains(marker),
            "cli.rs should not keep inline config layering helper: {marker}"
        );
    }
}

#[test]
fn cli_module_extraction_contract_keeps_helpers_in_new_module() {
    let config_layering_rs = read_repo_file("src/cli_config_layering.rs");
    assert!(
        config_layering_rs.contains("pub(super) fn build_layered_cli_args("),
        "cli_config_layering module should expose layered cli arg builder"
    );
    assert!(
        config_layering_rs.contains("fn map_config_entry_to_args("),
        "cli_config_layering module should own config entry-to-arg mapping"
    );
}
