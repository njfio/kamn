#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_METADATA_REPORT="$(mktemp)"
TMP_OUTPUT_DIR="$(mktemp -d)"
TMP_ERR="$(mktemp)"
TMP_REPO="$(mktemp -d)"
trap 'rm -f "$TMP_REPORT" "$TMP_METADATA_REPORT" "$TMP_ERR"; rm -rf "$TMP_OUTPUT_DIR" "$TMP_REPO"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork rust test matrix runner to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_rust_test_matrix_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork rust test matrix runner" >&2
  exit 1
fi

if ! grep -q "run_local_kolme_fork_rust_test_matrix_lane.sh" "$README_FILE"; then
  echo "expected README to reference local fork rust test matrix runner" >&2
  exit 1
fi

git -C "$TMP_REPO" init -q
git -C "$TMP_REPO" checkout -q -b main
git -C "$TMP_REPO" config user.email "ci@example.com"
git -C "$TMP_REPO" config user.name "CI Runner"
cat >"$TMP_REPO/README.md" <<'DOC'
local fork rust test matrix fixture
DOC
git -C "$TMP_REPO" add README.md
git -C "$TMP_REPO" commit -q -m "init fixture"
git -C "$TMP_REPO" remote add origin "https://github.com/njfio/kolme_fork.git"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path "$TMP_REPO" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --matrix-command "printf matrix_ok_1" \
    --matrix-command "printf matrix_ok_2" \
    --output-json "$TMP_REPORT" \
    --metadata-report "$TMP_METADATA_REPORT" \
    --command-output-dir "$TMP_OUTPUT_DIR"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run matrix lane to pass"
assert_eq "$(extract_value "$dry_run_output" "matrix_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget status"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-rust-test-matrix-summary.v1":
    raise SystemExit("unexpected local fork rust test matrix schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in local fork rust test matrix summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in local fork rust test matrix summary")
if report.get("command_count") != 2:
    raise SystemExit("expected command_count=2 for dry-run matrix")
checkpoints = report.get("checkpoints")
if not isinstance(checkpoints, list) or len(checkpoints) < 3:
    raise SystemExit("expected metadata + command checkpoint entries in matrix summary")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$TMP_REPO" \
  --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
  --expected-ref "refs/heads/main" \
  --matrix-command "printf matrix_ok" \
  --output-json "$TMP_REPORT" \
  --metadata-report "$TMP_METADATA_REPORT" \
  --command-output-dir "$TMP_OUTPUT_DIR" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected matrix lane run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for matrix run mode" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$TMP_REPO" \
      --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
      --expected-ref "refs/heads/main" \
      --matrix-command "printf matrix_ok_1" \
      --matrix-command "printf matrix_ok_2" \
      --max-seconds 20 \
      --output-json "$TMP_REPORT" \
      --metadata-report "$TMP_METADATA_REPORT" \
      --command-output-dir "$TMP_OUTPUT_DIR"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected matrix lane run mode to pass"
assert_eq "$(extract_value "$run_output" "matrix_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "fork_rust_test_matrix_passed" "expected pass reason code"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within-budget marker"

python3 - "$TMP_REPORT" "$TMP_OUTPUT_DIR" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
out_dir = pathlib.Path(sys.argv[2])
if report.get("mode") != "run":
    raise SystemExit("expected run mode in local fork rust test matrix summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status for run mode matrix summary")
if report.get("budget_status") != "within_budget":
    raise SystemExit("expected within_budget for run mode matrix summary")
log_files = sorted(out_dir.glob("command-*.log"))
if len(log_files) != 2:
    raise SystemExit("expected two command output files in matrix output directory")
for idx, marker in enumerate(("matrix_ok_1", "matrix_ok_2"), start=1):
    content = (out_dir / f"command-{idx}.log").read_text(encoding="utf-8")
    if marker not in content:
        raise SystemExit(f"expected marker {marker} in command-{idx}.log")
PY

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$TMP_REPO" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --matrix-command "sleep 2" \
    --max-seconds 1 \
    --output-json "$TMP_REPORT" \
    --metadata-report "$TMP_METADATA_REPORT" \
    --command-output-dir "$TMP_OUTPUT_DIR" >"$TMP_ERR" 2>&1
timeout_code=$?
set -e

if [ "$timeout_code" -eq 0 ]; then
  echo "expected matrix lane to fail when command exceeds max-seconds budget" >&2
  exit 1
fi

if ! grep -q "reason_code=fork_rust_test_command_timeout" "$TMP_ERR"; then
  echo "expected fork_rust_test_command_timeout reason marker for command timeout failure" >&2
  exit 1
fi

echo "local fork rust test matrix lane tests passed."
