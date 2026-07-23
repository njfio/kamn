use std::path::Path;

#[path = "../tests/support/service_authority_fixture.rs"]
mod service_authority_fixture;

pub(super) fn write(root: &Path, recipient: &str) {
    let path = root.join("staging/service-api-state.json");
    let state = service_authority_fixture::state(recipient);
    std::fs::write(path, serde_json::to_vec(&state).expect("state JSON"))
        .expect("persisted service state");
}
