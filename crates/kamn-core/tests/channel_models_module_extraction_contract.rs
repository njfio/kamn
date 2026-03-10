use std::fs;
use std::path::Path;

const ROOT: &str = "src/channel_models.rs";
const ROOT_BUDGET: usize = 220;
const REQUIRED_MODULE_MARKERS: &[&str] = &[
    "mod channel_errors;",
    "mod channel_store;",
    "mod channel_types;",
    "mod snapshot_codec;",
    "mod snapshot_store;",
];
const REQUIRED_FILES: &[&str] = &[
    "src/channel_models/channel_errors.rs",
    "src/channel_models/channel_store.rs",
    "src/channel_models/channel_types.rs",
    "src/channel_models/snapshot_codec.rs",
    "src/channel_models/snapshot_store.rs",
];
const MOVED_ROOT_SNIPPETS: &[&str] = &[
    "pub enum ChannelType {",
    "pub struct ChannelStore {",
    "pub enum ChannelModelError {",
    "pub trait ChannelSnapshotStore {",
    "fn serialize_channel_snapshot(",
    "fn parse_channel_snapshot_payload(",
];

#[test]
fn channel_models_root_is_extracted() {
    let source = fs::read_to_string(ROOT).expect("root channel models file should be readable");
    assert_root_budget(source.as_str());
    assert_root_markers(source.as_str());
    assert_extracted_files();
    assert_moved_surface(source.as_str());
}

fn assert_root_budget(source: &str) {
    let line_count = source.lines().count();
    assert!(
        line_count <= ROOT_BUDGET,
        "expected {ROOT} to be <= {ROOT_BUDGET} lines after extraction, got {line_count}"
    );
}

fn assert_root_markers(source: &str) {
    for marker in REQUIRED_MODULE_MARKERS {
        assert!(
            source.contains(marker),
            "expected root shell to contain module marker `{marker}`"
        );
    }
}

fn assert_extracted_files() {
    for path in REQUIRED_FILES {
        let file = Path::new(path);
        assert!(file.exists(), "expected extracted file `{path}` to exist");
        let line_count = fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("failed to read `{path}`: {error}"))
            .lines()
            .count();
        assert!(
            line_count <= 200,
            "expected extracted file `{path}` to stay within 200 lines, got {line_count}"
        );
    }
}

fn assert_moved_surface(source: &str) {
    for snippet in MOVED_ROOT_SNIPPETS {
        assert!(
            !source.contains(snippet),
            "expected root shell to move `{snippet}` into an extracted module"
        );
    }
}
