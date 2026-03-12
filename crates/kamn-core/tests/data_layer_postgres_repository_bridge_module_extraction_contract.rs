use std::fs;
use std::path::PathBuf;

const ROOT: &str = "src/data_layer_postgres_repository_bridge.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/data_layer_postgres_repository_bridge/models.rs",
    "src/data_layer_postgres_repository_bridge/session.rs",
    "src/data_layer_postgres_repository_bridge/m5_pgvector.rs",
    "src/data_layer_postgres_repository_bridge/m6_age.rs",
    "src/data_layer_postgres_repository_bridge/m7_timescale.rs",
    "src/data_layer_postgres_repository_bridge/support.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod m5_pgvector;",
    "mod m6_age;",
    "mod m7_timescale;",
    "mod models;",
    "mod session;",
    "mod support;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub enum DataLayerPgOperationKind {",
    "pub struct DataLayerPgRequesterSession {",
    "pub struct DataLayerPgSqlOperation {",
    "pub struct DataLayerPgM5PgvectorConfig {",
    "pub struct DataLayerPgM6AgeConfig {",
    "pub struct DataLayerPgM7TimescaleConfig {",
    "pub enum DataLayerPgRepositoryBridgeError {",
    "fn build_requester_session(",
    "fn validate_pgvector_extension(",
    "fn validate_age_config(",
    "fn validate_timescale_config(",
    "fn map_age_supported_relation(",
];

#[test]
fn data_layer_postgres_repository_bridge_root_is_extracted() {
    let root_text = read_root();
    assert_root_budget(&root_text);
    assert_root_markers(&root_text, REQUIRED_MARKERS);
    assert_root_excludes(&root_text, MOVED_MARKERS);
    assert_module_files_exist_and_fit();
}

fn read_root() -> String {
    fs::read_to_string(repo_root().join(ROOT)).expect("read bridge root")
}

fn assert_root_budget(root_text: &str) {
    let root_lines = line_count(root_text);
    assert!(
        root_lines <= ROOT_CAP,
        "expected {ROOT} to be <= {ROOT_CAP} lines, got {root_lines}"
    );
}

fn assert_root_markers(root_text: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            root_text.contains(marker),
            "expected root shell to contain marker `{marker}`"
        );
    }
}

fn assert_root_excludes(root_text: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            !root_text.contains(marker),
            "expected root shell to move marker `{marker}` into modules"
        );
    }
}

fn assert_module_files_exist_and_fit() {
    for module in MODULE_FILES {
        let module_path = repo_root().join(module);
        assert!(
            module_path.exists(),
            "expected module file `{module}` to exist"
        );
        let module_text = fs::read_to_string(&module_path).expect("read module file");
        let module_lines = line_count(&module_text);
        assert!(
            module_lines <= MODULE_CAP,
            "expected `{module}` to be <= {MODULE_CAP} lines, got {module_lines}"
        );
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn line_count(text: &str) -> usize {
    text.lines().count()
}
