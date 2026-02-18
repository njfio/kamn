#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_native_api_parity_live_proof_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_native_api_parity_live_proof_lane.json"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DEVNET_OPS_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
TMP_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local native API parity live proof lane runner to be executable" >&2
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
  echo "expected local native API parity live proof runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local native API parity live proof runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local native API parity live proof lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("expected local native API parity live proof lane manifest schema")
if payload.get("lane_id") != "kolme.local_native_api_parity_live_proof.run":
    raise SystemExit("expected local native API parity live proof lane manifest lane_id")
run_command = payload.get("phases", {}).get("run")
if run_command != [
    "bash",
    "scripts/kolme/run_local_native_api_parity_live_proof_lane_impl.sh",
]:
    raise SystemExit("expected local native API parity live proof lane manifest run command")
PY

manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
assert_eq "$manifest_path" "$MANIFEST" "expected local native API parity wrapper to resolve deterministic manifest"
if bash "$DISPATCHER" --lane-wrapper run_missing_native_api_parity_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected local run lane dispatcher to fail closed for unknown native parity wrapper" >&2
  exit 1
fi

if ! grep -q "run_local_native_api_parity_live_proof_lane.sh" "$DEVNET_OPS_DOC"; then
  echo "expected Kolme devnet ops doc to reference local native API parity live proof lane runner" >&2
  exit 1
fi

if ! grep -q "run_local_native_api_parity_live_proof_lane.sh" "$CI_STRATEGY_DOC"; then
  echo "expected CI strategy doc to reference local native API parity live proof lane runner" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run native parity lane to pass"
assert_eq "$(extract_value "$dry_run_output" "lane_mode")" "dry-run" "expected dry-run lane mode marker"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected dry-run reason marker"
assert_eq "$(extract_value "$dry_run_output" "budget_status")" "not_run" "expected dry-run budget marker"
assert_eq "$(extract_value "$dry_run_output" "local_only_enforced")" "true" "expected local-only marker for native parity lane"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.local-native-api-parity-live-proof-summary.v1":
    raise SystemExit("unexpected local native parity summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run native parity mode in summary")
if report.get("status") != "ok":
    raise SystemExit("expected ok status for dry-run native parity summary")
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code marker in native parity summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in native parity summary")
checks = report.get("checks")
if not isinstance(checks, list) or len(checks) < 3:
    raise SystemExit("expected deterministic check entries in native parity summary")
planned_ids = [entry.get("id") for entry in checks if entry.get("status") == "planned"]
for required in ("nonce_probe", "broadcast_probe", "finality_probe"):
    if required not in planned_ids:
        raise SystemExit(f"expected planned native parity check id: {required}")
PY

set +e
bash "$RUNNER" \
  --mode run \
  --nonce-command "printf '%s\n' nonce" \
  --broadcast-command "printf '%s\n' broadcast" \
  --finality-command "printf '%s\n' finality" \
  --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected native parity lane run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic opt-in failure message for native parity lane run mode" >&2
  exit 1
fi

echo "local native API parity live proof lane tests passed."
