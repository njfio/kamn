#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_SCRIPT="$ROOT_DIR/scripts/runtime/run_live_network_smoke_lane.sh"
SMOKE_SCRIPT_IMPL="$ROOT_DIR/scripts/runtime/run_live_network_smoke_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_live_network_smoke_lane.json"
EXEC_DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
EXEC_REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$SMOKE_SCRIPT" ]; then
  echo "expected live-network smoke lane runner to be executable" >&2
  exit 1
fi
if [ ! -x "$SMOKE_SCRIPT_IMPL" ]; then
  echo "expected live-network smoke lane implementation runner to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -x "$EXEC_DISPATCHER" ]; then
  echo "expected shared exec dispatcher to be executable" >&2
  exit 1
fi
if [ ! -f "$EXEC_REGISTRY" ]; then
  echo "expected exec wrapper registry to exist" >&2
  exit 1
fi

if [ ! -L "$SMOKE_SCRIPT" ]; then
  echo "expected live-network smoke lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi

if [ "$(readlink "$SMOKE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected live-network smoke lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$SMOKE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected live-network smoke lane wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_live_network_smoke_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected live-network smoke lane manifest to dispatch implementation module" >&2
  exit 1
fi

if [ ! -L "$SMOKE_SCRIPT_IMPL" ]; then
  echo "expected live-network smoke lane implementation wrapper to be a symlink" >&2
  exit 1
fi

if [ "$(readlink -f "$SMOKE_SCRIPT_IMPL")" != "$(readlink -f "$EXEC_DISPATCHER")" ]; then
  echo "expected live-network smoke lane implementation wrapper to resolve to shared exec dispatcher" >&2
  exit 1
fi

python3 - "$EXEC_REGISTRY" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
entry = registry.get("entries", {}).get("scripts/runtime/run_live_network_smoke_lane_impl.sh")
if not isinstance(entry, dict):
    raise SystemExit("expected registry entry for live-network smoke lane implementation wrapper")
if entry.get("interpreter") != "python3":
    raise SystemExit("expected python3 interpreter for live-network smoke lane implementation wrapper")
if entry.get("target") != "scripts/runtime/live_network_smoke_lane_contract.py":
    raise SystemExit("expected live-network smoke lane implementation wrapper target in exec registry")
if entry.get("args_prefix") != []:
    raise SystemExit("expected empty args_prefix for live-network smoke lane implementation wrapper")
if entry.get("passthrough") is not True:
    raise SystemExit("expected passthrough=true for live-network smoke lane implementation wrapper")
PY

smoke_output="$(
  bash "$SMOKE_SCRIPT" \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$smoke_output" | grep -q '^status=pass$'; then
  echo "expected live-network smoke lane to report pass status" >&2
  exit 1
fi
if ! printf '%s\n' "$smoke_output" | grep -q '^final_decision=GO$'; then
  echo "expected live-network smoke lane to report GO decision" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.runtime.live-network-smoke-report.v1":
    raise SystemExit("unexpected live-network smoke report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live-network smoke report final decision to be GO")
if payload.get("status") != "pass":
    raise SystemExit("expected live-network smoke report status to be pass")
if payload.get("command_count", 0) < 2:
    raise SystemExit("expected live-network smoke report to record at least two smoke commands")
PY

set +e
budget_failure_output="$(
  KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS=true \
  KAMN_LIVE_NETWORK_SMOKE_FAKE_DELAY_SECONDS=1 \
  KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS=0 \
  bash "$SMOKE_SCRIPT" \
    --output-json "$TMP_REPORT" 2>&1
)"
budget_failure_code=$?
set -e

if [ "$budget_failure_code" -eq 0 ]; then
  echo "expected live-network smoke lane budget guard to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$budget_failure_output" | grep -q "exceeded runtime budget"; then
  echo "expected budget-failure run to emit runtime budget guard message" >&2
  exit 1
fi

echo "live-network smoke lane script tests passed."
