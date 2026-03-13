use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("read source file")
}

fn lines(path: &str) -> usize {
    read(path).lines().count()
}

#[test]
fn regression_tcp_root_is_split_into_bounded_modules() {
    let root = "crates/kamn-sdk/src/tcp.rs";
    let extracted = [
        "crates/kamn-sdk/src/tcp/envelope.rs",
        "crates/kamn-sdk/src/tcp/handshake.rs",
        "crates/kamn-sdk/src/tcp/transport.rs",
        "crates/kamn-sdk/src/tcp/support.rs",
        "crates/kamn-sdk/src/tcp/tests.rs",
    ];
    let root_contents = read(root);

    assert!(
        lines(root) <= 180,
        "expected {root} to be <= 180 LOC after extraction, got {}",
        lines(root)
    );

    for marker in [
        "#[path = \"tcp/envelope.rs\"]",
        "#[path = \"tcp/handshake.rs\"]",
        "#[path = \"tcp/transport.rs\"]",
        "#[path = \"tcp/support.rs\"]",
        "#[cfg(test)]",
        "#[path = \"tcp/tests.rs\"]",
    ] {
        assert!(
            root_contents.contains(marker),
            "expected root shell to contain marker: {marker}"
        );
    }

    for path in extracted {
        assert!(repo_root().join(path).is_file(), "expected extracted file {path} to exist");
        assert!(lines(path) <= 200, "expected {path} to be <= 200 LOC, got {}", lines(path));
    }
}
