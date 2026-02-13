#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_fork_sync_metadata_lane.sh"
RUNNER_IMPL="$ROOT_DIR/scripts/kolme/run_local_fork_sync_metadata_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_fork_sync_metadata_lane.json"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
TMP_REPO="$(mktemp -d)"
trap 'rm -f "$TMP_REPORT" "$TMP_ERR"; rm -rf "$TMP_REPO"' EXIT

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
  echo "expected local fork sync metadata runner to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNNER_IMPL" ]; then
  echo "expected local fork sync metadata implementation runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected local fork sync metadata runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local fork sync metadata runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork sync metadata lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local fork sync metadata lane manifest schema")
if payload.get("lane_id") != "kolme.local_fork_sync_metadata.run":
    raise SystemExit("expected local fork sync metadata lane manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_fork_sync_metadata_lane_impl.sh",
]:
    raise SystemExit("expected local fork sync metadata lane manifest run command")
PY

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
assert_eq "$manifest_path" "$MANIFEST" "expected local fork sync metadata wrapper to resolve deterministic manifest"
if bash "$DISPATCHER" --lane-wrapper run_missing_local_fork_sync_metadata_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown local fork sync metadata wrapper" >&2
  exit 1
fi

if ! grep -q "run_local_fork_sync_metadata_lane.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local fork sync metadata runner" >&2
  exit 1
fi

git -C "$TMP_REPO" init -q
git -C "$TMP_REPO" checkout -q -b main
git -C "$TMP_REPO" config user.email "ci@example.com"
git -C "$TMP_REPO" config user.name "CI Runner"
cat >"$TMP_REPO/README.md" <<'EOF'
local fork sync metadata test fixture
EOF
git -C "$TMP_REPO" add README.md
git -C "$TMP_REPO" commit -q -m "init metadata fixture"
git -C "$TMP_REPO" remote add origin "https://github.com/njfio/kolme_fork.git"
EXPECTED_COMMIT="$(git -C "$TMP_REPO" rev-parse HEAD)"

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --checkout-path "$TMP_REPO" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --expected-commit "$EXPECTED_COMMIT" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run metadata sync to pass"
assert_eq "$(extract_value "$dry_run_output" "sync_mode")" "dry-run" "expected dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason code"
assert_eq "$(extract_value "$dry_run_output" "metadata_verified")" "false" "expected dry-run metadata verification marker"

python3 - "$TMP_REPORT" "$EXPECTED_COMMIT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-fork-sync-metadata-summary.v1":
    raise SystemExit("unexpected metadata sync summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run mode in metadata sync summary")
if report.get("expected_commit") != sys.argv[2]:
    raise SystemExit("expected expected_commit marker in metadata sync summary")
checks = report.get("checks")
if not isinstance(checks, list) or len(checks) < 4:
    raise SystemExit("expected deterministic metadata sync checks in summary")
if not any(entry.get("id") == "origin_remote_matches" for entry in checks if isinstance(entry, dict)):
    raise SystemExit("expected origin_remote_matches check id in metadata summary")
PY

run_output="$(
  bash "$RUNNER" \
    --mode run \
    --checkout-path "$TMP_REPO" \
    --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
    --expected-ref "refs/heads/main" \
    --expected-commit "$EXPECTED_COMMIT" \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$run_output" "status")" "ok" "expected metadata sync run mode to pass for matching repo/ref"
assert_eq "$(extract_value "$run_output" "sync_mode")" "run" "expected run mode marker"
assert_eq "$(extract_value "$run_output" "reason_code")" "fork_metadata_verified" "expected verified reason code"
assert_eq "$(extract_value "$run_output" "metadata_verified")" "true" "expected metadata verification marker"

python3 - "$TMP_REPORT" "$EXPECTED_COMMIT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
metadata = report.get("metadata", {})
if report.get("mode") != "run":
    raise SystemExit("expected run mode in metadata sync summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status for matching metadata run")
if metadata.get("head_ref") != "refs/heads/main":
    raise SystemExit("expected refs/heads/main in metadata head_ref")
if metadata.get("dirty_checkout") is not False:
    raise SystemExit("expected clean checkout metadata")
if not metadata.get("head_commit"):
    raise SystemExit("expected non-empty head_commit metadata")
if report.get("expected_commit") != sys.argv[2]:
    raise SystemExit("expected expected_commit marker in run metadata sync summary")
if metadata.get("head_commit") != report.get("expected_commit"):
    raise SystemExit("expected head_commit to match expected_commit in metadata sync summary")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$TMP_REPO" \
  --expected-remote-url "https://github.com/njfio/kolme_fork.git" \
  --expected-ref "refs/heads/main" \
  --expected-commit "0000000000000000000000000000000000000000" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_commit_mismatch_code=$?
set -e

if [ "$run_commit_mismatch_code" -eq 0 ]; then
  echo "expected metadata sync run mode to fail on pinned commit mismatch" >&2
  exit 1
fi

if ! grep -q "reason_code=head_commit_mismatch" "$TMP_ERR"; then
  echo "expected head_commit_mismatch reason marker for pinned commit mismatch failure" >&2
  exit 1
fi

set +e
bash "$RUNNER" \
  --mode run \
  --checkout-path "$TMP_REPO" \
  --expected-remote-url "https://github.com/fpco/kolme.git" \
  --expected-ref "refs/heads/main" \
  --expected-commit "$EXPECTED_COMMIT" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_mismatch_code=$?
set -e

if [ "$run_mismatch_code" -eq 0 ]; then
  echo "expected metadata sync run mode to fail on remote URL mismatch" >&2
  exit 1
fi

if ! grep -q "reason_code=remote_url_mismatch" "$TMP_ERR"; then
  echo "expected remote_url_mismatch reason marker for URL mismatch failure" >&2
  exit 1
fi

echo "local fork sync metadata lane tests passed."
