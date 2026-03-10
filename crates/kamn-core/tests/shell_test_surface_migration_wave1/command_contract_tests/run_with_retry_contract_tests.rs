use super::super::support::*;

const FLAKY_ONCE_SCRIPT: &str = "#!/usr/bin/env bash\nset -euo pipefail\nf=\"$1\"\nif [ ! -f \"$f\" ]; then\n  echo 1 > \"$f\"\n  exit 1\nfi\nexit 0\n";

#[test]
fn spec_c10_run_with_retry_contract() {
    let script = repo_path("scripts/ci/run_with_retry.sh");
    assert!(script.is_file(), "run_with_retry helper must exist");

    let tmp = TempDir::new("run-with-retry");
    assert_immediate_success(&script, tmp.path());
    assert_flaky_once_success(&script, tmp.path());
    assert_always_fail_case(&script);
}

fn run_with_retry(
    script: &Path,
    label: &str,
    command_text: &str,
    output_file: Option<&Path>,
) -> CommandOutput {
    let mut command = Command::new("bash");
    populate_retry_command(&mut command, script, label, command_text, output_file);
    run_command(command, &format!("run_with_retry {label} case"))
}

fn populate_retry_command(
    command: &mut Command,
    script: &Path,
    label: &str,
    command_text: &str,
    output_file: Option<&Path>,
) {
    command
        .arg(script)
        .arg("--label")
        .arg(label)
        .arg("--max-attempts")
        .arg("2")
        .arg("--")
        .arg("bash")
        .arg("-lc")
        .arg(command_text);
    if let Some(output_file) = output_file {
        command.env("GITHUB_OUTPUT", output_file);
    }
}

fn assert_retry_output(output_file: &Path, expected: &[&str], label: &str) {
    let markers = fs::read_to_string(output_file).expect("failed to read run_with_retry output");
    assert_contains_all(&markers, expected, label);
}

fn assert_immediate_success(script: &Path, tmp_root: &Path) {
    let output_file = tmp_root.join("out_success.txt");
    let output = run_with_retry(script, "immediate", "exit 0", Some(&output_file));
    assert_success(&output, "run_with_retry immediate-success case");
    assert_retry_output(
        &output_file,
        &[
            "retry_attempts<<EOF\n1\nEOF",
            "retry_used<<EOF\nfalse\nEOF",
            "retry_final_status<<EOF\npassed\nEOF",
        ],
        "run_with_retry immediate-success output markers",
    );
}

fn assert_flaky_once_success(script: &Path, tmp_root: &Path) {
    let flaky_once = tmp_root.join("flaky_once.sh");
    let counter_file = tmp_root.join("counter");
    let output_file = tmp_root.join("out_retry.txt");
    fs::write(&flaky_once, FLAKY_ONCE_SCRIPT).expect("failed to write flaky_once script");
    let command_text = format!(
        "bash '{}' '{}'",
        flaky_once.display(),
        counter_file.display()
    );
    let output = run_with_retry(script, "flaky-once", &command_text, Some(&output_file));
    assert_success(&output, "run_with_retry flaky-once case");
    assert_retry_output(
        &output_file,
        &[
            "retry_attempts<<EOF\n2\nEOF",
            "retry_used<<EOF\ntrue\nEOF",
            "retry_final_status<<EOF\npassed\nEOF",
        ],
        "run_with_retry flaky-once output markers",
    );
}

fn assert_always_fail_case(script: &Path) {
    let output = run_with_retry(script, "always-fail", "exit 1", None);
    assert_failure(&output, "run_with_retry always-fail case");
}
