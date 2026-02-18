#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_fork_smoke_evidence_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_fork_smoke_evidence_lane.json"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
TMP_METADATA_REPORT="$(mktemp)"
TMP_SMOKE_OUTPUT="$(mktemp)"
TMP_ERR="$(mktemp)"
TMP_REPO="$(mktemp -d)"
trap 'rm -f "$TMP_REPORT" "$TMP_METADATA_REPORT" "$TMP_SMOKE_OUTPUT" "$TMP_ERR"; rm -rf "$TMP_REPO"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork smoke evidence runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected local fork smoke evidence runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local fork smoke evidence runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork smoke evidence lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local fork smoke evidence lane manifest schema")
if payload.get("lane_id") != "kolme.local_fork_smoke_evidence.run":
    raise SystemExit("expected local fork smoke evidence lane manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_fork_smoke_evidence_lane_impl.sh",
]:
    raise SystemExit("expected local fork smoke evidence lane manifest run command")
PY

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
assert_eq "$manifest_path" "$MANIFEST" "expected local fork smoke evidence wrapper to resolve deterministic manifest"
if bash "$DISPATCHER" --lane-wrapper run_missing_local_fork_smoke_evidence_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown local fork smoke evidence wrapper" >&2
  exit 1
fi

if ! grep -q "run_local_fork_smoke_evidence_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork smoke evidence runner" >&2
  exit 1
fi

git -C "$TMP_REPO" init -q
git -C "$TMP_REPO" checkout -q -b main
git -C "$TMP_REPO" config user.email "ci@example.com"
git -C "$TMP_REPO" config user.name "CI Runner"
cat >"$TMP_REPO/README.md" <<'EOF'
local fork smoke evidence test fixture
EOF
git -C "$TMP_REPO" add README.md
git -C "$TMP_REPO" commit -q -m "init smoke fixture"
git -C "$TMP_REPO" remote add origin "https://github.com/njfio/kolme_fork.git"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path "$TMP_REPO" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --smoke-command "printf smoke_ok" \
    --output-json "$TMP_REPORT" \
    --metadata-report "$TMP_METADATA_REPORT" \
    --smoke-output-file "$TMP_SMOKE_OUTPUT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run smoke lane to pass"
assert_eq "$(extract_value "$dry_run_output" "smoke_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget status"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-smoke-evidence-summary.v1":
    raise SystemExit("unexpected local fork smoke summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in local fork smoke summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in local fork smoke summary")
if report.get("budget_status") != "not_run":
    raise SystemExit("expected not_run budget status for dry-run summary")
checkpoints = report.get("checkpoints")
if not isinstance(checkpoints, list) or len(checkpoints) < 2:
    raise SystemExit("expected checkpoint entries in local fork smoke summary")
if not any(entry.get("id") == "fork_sync_metadata" for entry in checkpoints if isinstance(entry, dict)):
    raise SystemExit("expected fork_sync_metadata checkpoint in local fork smoke summary")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$TMP_REPO" \
  --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
  --expected-ref "refs/heads/main" \
  --smoke-command "printf smoke_ok" \
  --output-json "$TMP_REPORT" \
  --metadata-report "$TMP_METADATA_REPORT" \
  --smoke-output-file "$TMP_SMOKE_OUTPUT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected smoke lane run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic local-only opt-in failure message for smoke run mode" >&2
  exit 1
fi

run_output="$(
  KAMN_KOLME_LOCAL_HEAVY=1 \
    bash "$RUNNER" \
      --mode run \
      --checkout-path "$TMP_REPO" \
      --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
      --expected-ref "refs/heads/main" \
      --smoke-command "printf smoke_ok" \
      --max-seconds 30 \
      --output-json "$TMP_REPORT" \
      --metadata-report "$TMP_METADATA_REPORT" \
      --smoke-output-file "$TMP_SMOKE_OUTPUT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected smoke lane run mode to pass"
assert_eq "$(extract_value "$run_output" "smoke_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "fork_smoke_command_passed" "expected pass reason code for run mode"
assert_eq "$(extract_value "$run_output" "budget_status")" "within_budget" "expected within_budget marker"

python3 - "$TMP_REPORT" "$TMP_SMOKE_OUTPUT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
smoke_output = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
if report.get("mode") != "run":
    raise SystemExit("expected run mode in local fork smoke summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status in local fork smoke run summary")
if report.get("budget_status") != "within_budget":
    raise SystemExit("expected within_budget status in local fork smoke run summary")
if report.get("max_seconds") != 30:
    raise SystemExit("expected max_seconds=30 in local fork smoke run summary")
if "smoke_ok" not in smoke_output:
    raise SystemExit("expected smoke command output marker")
PY

set +e
KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$TMP_REPO" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --smoke-command "sleep 2" \
    --max-seconds 1 \
    --output-json "$TMP_REPORT" \
    --metadata-report "$TMP_METADATA_REPORT" \
    --smoke-output-file "$TMP_SMOKE_OUTPUT" >"$TMP_ERR" 2>&1
timeout_code=$?
set -e

if [ "$timeout_code" -eq 0 ]; then
  echo "expected smoke lane to fail when smoke command exceeds max-seconds budget" >&2
  exit 1
fi

if ! grep -q "reason_code=fork_smoke_command_timeout" "$TMP_ERR"; then
  echo "expected fork_smoke_command_timeout reason marker for budget timeout failure" >&2
  exit 1
fi

echo "local fork smoke evidence lane tests passed."
