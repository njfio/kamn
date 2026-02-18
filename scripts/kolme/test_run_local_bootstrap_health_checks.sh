#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"
ROOT_DIR="$KAMN_ROOT"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_bootstrap_health_checks.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_bootstrap_health_checks_lane.json"
TMP_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_ERR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected Kolme local bootstrap health-check runner to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected local run lane dispatcher to be executable" >&2
  exit 1
fi
if [ ! -f "$MANIFEST" ]; then
  echo "expected local bootstrap health-check run-lane manifest to exist" >&2
  exit 1
fi
if [ ! -L "$RUNNER" ]; then
  echo "expected local bootstrap health-check runner to be a symlink to shared runtime lane dispatcher" >&2
  exit 1
fi
if [ "$(readlink "$RUNNER")" != "run_lane_dispatch.sh" ]; then
  echo "expected local bootstrap health-check runner symlink target to be run_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper run_local_bootstrap_health_checks.sh --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected bootstrap health-check wrapper to resolve deterministic run-lane manifest path" >&2
  exit 1
fi

# Regression: #1585
if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if ! grep -q "run_local_bootstrap_health_checks.sh" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference local bootstrap health-check runner" >&2
  exit 1
fi
if ! grep -q "run_lane_dispatch.sh --lane-wrapper run_local_bootstrap_health_checks.sh --resolve-manifest-path" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to reference bootstrap run-wrapper dispatcher mapping" >&2
  exit 1
fi

dry_run_output="$(
  bash "$RUNNER" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"

assert_eq "$(extract_value "$dry_run_output" "status")" "ok" "expected dry-run bootstrap run to pass"
assert_eq "$(extract_value "$dry_run_output" "bootstrap_mode")" "dry-run" "expected bootstrap dry-run mode marker"
assert_eq "$(extract_value "$dry_run_output" "readiness_status")" "planned" "expected planned readiness marker in dry-run"
assert_eq "$(extract_value "$dry_run_output" "reason_code")" "dry_run_no_commands_executed" "expected deterministic dry-run reason code"

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.kolme.local-bootstrap-summary.v1":
    raise SystemExit("unexpected bootstrap summary schema")
if report.get("mode") != "dry-run":
    raise SystemExit("expected dry-run bootstrap mode in summary")
if report.get("ready") is not False:
    raise SystemExit("expected ready=false for bootstrap dry-run summary")
if report.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code in bootstrap summary")
if report.get("local_only_enforced") is not True:
    raise SystemExit("expected local_only_enforced=true in bootstrap summary")
checks = report.get("checks")
if not isinstance(checks, list) or len(checks) < 3:
    raise SystemExit("expected bootstrap summary to include deterministic check entries")
if not any(entry.get("id") == "triadic_devnet_smoke" for entry in checks if isinstance(entry, dict)):
    raise SystemExit("expected bootstrap summary to include triadic_devnet_smoke check id")
PY

set +e
bash "$RUNNER" --mode run --output-json "$TMP_REPORT" >"$TMP_ERR" 2>&1
run_without_opt_in_code=$?
set -e

# Regression: #1417
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected bootstrap run mode without opt-in to fail closed" >&2
  exit 1
fi

if ! grep -q "requires explicit local-only opt-in" "$TMP_ERR"; then
  echo "expected deterministic opt-in failure message for bootstrap run mode" >&2
  exit 1
fi

echo "Kolme local bootstrap health-check runner tests passed."
