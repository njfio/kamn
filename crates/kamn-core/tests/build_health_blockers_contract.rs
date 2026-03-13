use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).expect("read source")
}

#[test]
fn regression_build_health_blockers_are_removed() {
    let batch = read("crates/kamn-core/src/data_layer_m1/batch.rs");
    assert!(
        !batch.contains("levels.last().unwrap()") && !batch.contains("levels.last().unwrap()[0]"),
        "batch assembly should not use production unwrap()"
    );

    let m7_tests = read("crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs");
    assert!(
        m7_tests.contains("#[cfg(test)]") && m7_tests.contains("mod tests {") && m7_tests.trim_end().ends_with('}'),
        "m7 telemetry tests should be wrapped in a real cfg(test) module"
    );
}
