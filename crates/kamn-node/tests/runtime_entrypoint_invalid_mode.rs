use std::process::Command;

#[test]
fn regression_runtime_entrypoint_rejects_invalid_runtime_mode_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_kamn-node"))
        .args(["--runtime-mode", "invalid-mode"])
        .output()
        .expect("kamn-node binary should execute");

    assert!(
        !output.status.success(),
        "invalid runtime mode must fail closed with non-zero exit status"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid runtime mode: invalid-mode"),
        "stderr must include deterministic invalid runtime-mode error text"
    );
}
