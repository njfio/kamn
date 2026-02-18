#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/ci/run_daemon_os_signal_reproducer.sh"

test_harness_require_executable "$SCRIPT" "expected daemon os-signal reproducer script to be executable"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cat >"$TMP_DIR/mock-cargo.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
test_name=""
for arg in "$@"; do
  case "$arg" in
    *::*)
      test_name="$arg"
      ;;
  esac
done
if [ -z "$test_name" ]; then
  echo "mock cargo expected rust test identifier argument" >&2
  exit 2
fi
echo "test result: ok"
exit 0
EOF
chmod +x "$TMP_DIR/mock-cargo.sh"

cat >"$TMP_DIR/mock-cargo-flaky.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
test_name=""
for arg in "$@"; do
  case "$arg" in
    *::*)
      test_name="$arg"
      ;;
  esac
done
if [ -z "$test_name" ]; then
  echo "mock cargo expected rust test identifier argument" >&2
  exit 2
fi
if [ "$test_name" = "main_tests::runtime_tests::regression_runtime_full_os_signal_stop_markers_project_shutdown_field_parity" ]; then
  echo "test result: FAILED" >&2
  echo "panic: forced failure for $test_name" >&2
  exit 1
fi
echo "test result: ok"
exit 0
EOF
chmod +x "$TMP_DIR/mock-cargo-flaky.sh"

stable_artifact_dir="$TMP_DIR/stable-artifacts"
stable_report="$TMP_DIR/stable-report.json"
stable_output="$(
  bash "$SCRIPT" \
    --attempts 2 \
    --max-seconds 30 \
    --artifact-dir "$stable_artifact_dir" \
    --output-json "$stable_report" \
    --cargo-bin "$TMP_DIR/mock-cargo.sh"
)"

for marker in \
  '^daemon_os_signal_reproducer_status=pass$' \
  '^daemon_os_signal_reproducer_reason_code=stable_success$' \
  '^daemon_os_signal_reproducer_final_decision=GO$'; do
  if ! printf '%s\n' "$stable_output" | grep -q "$marker"; then
    echo "expected stable daemon reproducer marker: $marker" >&2
    exit 1
  fi
done

python3 - "$stable_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.daemon-os-signal-reproducer-report.v1":
    raise SystemExit("unexpected daemon os-signal reproducer schema")
if report.get("status") != "pass":
    raise SystemExit("expected daemon os-signal stable status=pass")
if report.get("final_decision") != "GO":
    raise SystemExit("expected daemon os-signal stable final_decision=GO")
if report.get("reason_code") != "stable_success":
    raise SystemExit("expected daemon os-signal stable reason_code=stable_success")
runs = report.get("runs")
if not isinstance(runs, list) or len(runs) != 6:
    raise SystemExit("expected six daemon os-signal run entries for two attempts")
first = runs[0]
required_keys = {
    "test_name",
    "attempt_index",
    "run_index",
    "status",
    "exit_code",
    "log_file",
    "stdout_excerpt",
    "failure_markers",
}
missing = required_keys - set(first.keys())
if missing:
    raise SystemExit(f"missing daemon os-signal run keys: {sorted(missing)}")
if any(entry.get("status") != "pass" for entry in runs):
    raise SystemExit("expected all stable daemon os-signal runs to pass")
PY

flaky_artifact_dir="$TMP_DIR/flaky-artifacts"
flaky_report="$TMP_DIR/flaky-report.json"
flaky_output_file="$TMP_DIR/flaky-output.log"
set +e
bash "$SCRIPT" \
  --attempts 2 \
  --max-seconds 30 \
  --artifact-dir "$flaky_artifact_dir" \
  --output-json "$flaky_report" \
  --cargo-bin "$TMP_DIR/mock-cargo-flaky.sh" >"$flaky_output_file" 2>&1
flaky_status=$?
set -e
flaky_output="$(cat "$flaky_output_file")"
if [ "$flaky_status" -eq 0 ]; then
  echo "expected daemon os-signal reproducer to fail closed when one test fails" >&2
  exit 1
fi
for marker in \
  '^daemon_os_signal_reproducer_status=fail$' \
  '^daemon_os_signal_reproducer_final_decision=NO-GO$' \
  '^daemon_os_signal_reproducer_reason_code=flaky_pattern_observed$'; do
  if ! printf '%s\n' "$flaky_output" | grep -q "$marker"; then
    echo "expected flaky daemon reproducer marker: $marker" >&2
    exit 1
  fi
done

python3 - "$flaky_report" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.daemon-os-signal-reproducer-report.v1":
    raise SystemExit("unexpected daemon os-signal reproducer schema")
if report.get("status") != "fail":
    raise SystemExit("expected daemon os-signal flaky status=fail")
if report.get("final_decision") != "NO-GO":
    raise SystemExit("expected daemon os-signal flaky final_decision=NO-GO")
if report.get("reason_code") != "flaky_pattern_observed":
    raise SystemExit("expected daemon os-signal flaky reason_code=flaky_pattern_observed")
runs = report.get("runs")
if not isinstance(runs, list) or len(runs) != 6:
    raise SystemExit("expected six daemon os-signal run entries")
failed = [entry for entry in runs if entry.get("status") == "fail"]
if len(failed) != 2:
    raise SystemExit("expected exactly two failed daemon os-signal runs")
for entry in failed:
    markers = entry.get("failure_markers")
    if not isinstance(markers, list) or not markers:
        raise SystemExit("expected failure markers for failed daemon os-signal run")
PY

set +e
invalid_attempt_output="$(
  bash "$SCRIPT" \
    --attempts 0 \
    --max-seconds 30 \
    --artifact-dir "$TMP_DIR/invalid-artifacts" \
    --output-json "$TMP_DIR/invalid-report.json" \
    --cargo-bin "$TMP_DIR/mock-cargo.sh" 2>&1
)"
invalid_attempt_status=$?
set -e
if [ "$invalid_attempt_status" -eq 0 ]; then
  echo "expected attempts=0 validation failure for daemon os-signal reproducer" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_attempt_output" | grep -q "attempts must be greater than zero"; then
  echo "expected deterministic daemon reproducer attempts validation marker" >&2
  exit 1
fi

echo "run_daemon_os_signal_reproducer tests passed."
