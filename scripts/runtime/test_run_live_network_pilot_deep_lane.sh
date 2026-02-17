#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_lane.sh"
DEEP_LANE_IMPL="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_live_network_pilot_deep_lane.json"
EXEC_DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
EXEC_REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -x "$DEEP_LANE" ]]; then
  echo "expected live-network pilot deep lane script to be executable" >&2
  exit 1
fi
if [[ ! -x "$DEEP_LANE_IMPL" ]]; then
  echo "expected live-network pilot deep lane implementation script to be executable" >&2
  exit 1
fi
if [[ ! -x "$DISPATCHER" ]]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [[ ! -x "$EXEC_DISPATCHER" ]]; then
  echo "expected shared exec dispatcher to be executable" >&2
  exit 1
fi
if [[ ! -f "$EXEC_REGISTRY" ]]; then
  echo "expected exec wrapper registry to exist" >&2
  exit 1
fi

if [[ ! -L "$DEEP_LANE" ]]; then
  echo "expected live-network pilot deep lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [[ "$(readlink "$DEEP_LANE")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]]; then
  echo "expected live-network pilot deep lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$DEEP_LANE")" --resolve-manifest-path)"
if [[ "$resolved_manifest" != "$MANIFEST_FILE" ]]; then
  echo "expected live-network pilot deep lane wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_live_network_pilot_deep_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected live-network pilot deep lane manifest to dispatch implementation module" >&2
  exit 1
fi

if [[ ! -L "$DEEP_LANE_IMPL" ]]; then
  echo "expected live-network pilot deep lane implementation wrapper to be a symlink" >&2
  exit 1
fi

if [[ "$(readlink -f "$DEEP_LANE_IMPL")" != "$(readlink -f "$EXEC_DISPATCHER")" ]]; then
  echo "expected live-network pilot deep lane implementation wrapper to resolve to shared exec dispatcher" >&2
  exit 1
fi

python3 - "$EXEC_REGISTRY" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
entry = registry.get("entries", {}).get("scripts/runtime/run_live_network_pilot_deep_lane_impl.sh")
if not isinstance(entry, dict):
    raise SystemExit("expected registry entry for live-network pilot deep lane implementation wrapper")
if entry.get("interpreter") != "python3":
    raise SystemExit("expected python3 interpreter for live-network pilot deep lane implementation wrapper")
if entry.get("target") != "scripts/runtime/live_network_pilot_deep_lane_contract.py":
    raise SystemExit("expected live-network pilot deep lane implementation wrapper target in exec registry")
if entry.get("args_prefix") != []:
    raise SystemExit("expected empty args_prefix for live-network pilot deep lane implementation wrapper")
if entry.get("passthrough") is not True:
    raise SystemExit("expected passthrough=true for live-network pilot deep lane implementation wrapper")
PY

report_json="$TMP_DIR/live-network-pilot-deep-summary.json"
lane_output="$(
  bash "$DEEP_LANE" \
    --event-name schedule \
    --skip-suite \
    --max-seconds 120 \
    --output-json "$report_json"
)"

if ! printf '%s\n' "$lane_output" | grep -q "live-network pilot deep lane tests passed."; then
  echo "expected live-network pilot deep lane success output" >&2
  exit 1
fi

python3 - "$report_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-network-pilot-artifact-summary.v1":
    raise SystemExit("unexpected live-network pilot deep summary schema")
if payload.get("event_name") != "schedule":
    raise SystemExit("expected schedule event in live-network pilot deep summary")
if payload.get("cadence") != "scheduled":
    raise SystemExit("expected scheduled cadence in live-network pilot deep summary")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for live-network pilot deep summary")
PY

set +e
invalid_event_output="$(
  bash "$DEEP_LANE" \
    --event-name pull_request \
    --output-json "$TMP_DIR/invalid-cadence.json" 2>&1
)"
invalid_event_code=$?
set -e

if [[ "$invalid_event_code" -eq 0 ]]; then
  echo "expected live-network pilot deep lane to reject pull_request cadence" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_event_output" | grep -q "scheduled/manual-only cadence policy"; then
  echo "expected cadence policy rejection marker for live-network pilot deep lane" >&2
  exit 1
fi

set +e
invalid_skip_output="$(
  KAMN_LIVE_NETWORK_PILOT_DEEP_SMOKE_SKIP_COMMANDS=maybe \
  bash "$DEEP_LANE" \
    --event-name schedule \
    --skip-suite \
    --output-json "$TMP_DIR/invalid-smoke-skip-flag.json" 2>&1
)"
invalid_skip_code=$?
set -e

if [[ "$invalid_skip_code" -eq 0 ]]; then
  echo "expected live-network pilot deep lane to reject invalid smoke skip flag value" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_skip_output" | grep -q "KAMN_LIVE_NETWORK_PILOT_DEEP_SMOKE_SKIP_COMMANDS must be true or false"; then
  echo "expected explicit invalid smoke skip flag marker for live-network pilot deep lane" >&2
  exit 1
fi

echo "live-network pilot deep lane script tests passed."
