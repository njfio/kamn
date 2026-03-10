#[path = "support/channel_store.rs"]
mod channel_store;
#[path = "support/fault_io.rs"]
mod fault_io;
#[path = "support/fixture_parser.rs"]
mod fixture_parser;
#[path = "support/message_store.rs"]
mod message_store;
#[path = "support/snapshot_fixtures.rs"]
mod snapshot_fixtures;
#[path = "support/task_store.rs"]
mod task_store;
#[path = "support/temp_dir.rs"]
mod temp_dir;

pub(crate) use fixture_parser::{parse_case_line, parse_fixture, FixtureCase, FixtureMetadata};
pub(crate) use temp_dir::TempDir;

pub(crate) fn run_case(case: &FixtureCase, temp_dir: &TempDir) {
    match case.store.as_str() {
        "channel" => channel_store::run(case, temp_dir),
        "message_lifecycle" => message_store::run(case, temp_dir),
        "task_operation" => task_store::run(case, temp_dir),
        unknown => panic!("unknown fixture store row: {unknown}"),
    }
}
