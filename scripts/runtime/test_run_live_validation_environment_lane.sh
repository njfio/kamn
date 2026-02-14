#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_live_validation_environment_lane.sh"
LANE_IMPL_SCRIPT="$ROOT_DIR/scripts/runtime/run_live_validation_environment_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_live_validation_environment_lane.json"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$TMP_DIR/live-validation-environment-summary.json"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected live validation environment lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_IMPL_SCRIPT" ]; then
  echo "expected live validation environment lane implementation runner to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected live validation environment lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected live validation environment lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected live validation environment lane wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_live_validation_environment_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected live validation environment lane manifest to dispatch implementation module" >&2
  exit 1
fi
if ! grep -q 'live_validation_environment_lane_contract.py' "$LANE_IMPL_SCRIPT"; then
  echo "expected live validation environment lane implementation to delegate to environment lane contract module" >&2
  exit 1
fi

lane_output="$(
  bash "$LANE_SCRIPT" \
    --mode dry-run \
    --max-seconds 120 \
    --topology-max-seconds 60 \
    --kolme-max-seconds 120 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected live validation environment lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected live validation environment lane GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected live validation environment lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^topology_contract_status=verified$'; then
  echo "expected live validation environment lane topology marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_connectivity_contract_status=verified$'; then
  echo "expected live validation environment lane kolme connectivity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected live validation environment lane fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-validation-environment-report.v1":
    raise SystemExit("unexpected live validation environment lane schema")
if payload.get("status") != "pass":
    raise SystemExit("expected live validation environment lane status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live validation environment lane final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected live validation environment lane lane_mode=dry-run")
if payload.get("topology_contract_status") != "verified":
    raise SystemExit("expected topology_contract_status=verified")
if payload.get("kolme_connectivity_contract_status") != "verified":
    raise SystemExit("expected kolme_connectivity_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
commands = payload.get("commands")
if commands != [
    "scripts/deploy/validate_deployment_assets_live.sh",
    "scripts/kolme/run_local_live_node_validation_bundle_lane.sh",
]:
    raise SystemExit("unexpected command sequence in live validation environment lane report")
PY

set +e
invalid_budget_output="$(
  bash "$LANE_SCRIPT" \
    --mode dry-run \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected live validation environment lane to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_LIVE_VALIDATION_ENV_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for live validation environment lane" >&2
  exit 1
fi

set +e
missing_opt_in_output="$(
  bash "$LANE_SCRIPT" \
    --mode run \
    --max-seconds 120 \
    --topology-max-seconds 60 \
    --kolme-max-seconds 120 2>&1
)"
missing_opt_in_code=$?
set -e
if [ "$missing_opt_in_code" -eq 0 ]; then
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_KOLME_LOCAL_HEAVY=1'; then
  echo "expected deterministic opt-in marker for run mode" >&2
  exit 1
fi

echo "live validation environment lane script tests passed."
