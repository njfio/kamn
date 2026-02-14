#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh"
LANE_IMPL_SCRIPT="$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_go_no_go_gate_lane.json"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$TMP_DIR/go-no-go-gate-report.json"
TMP_FAULT_REPORT="$TMP_DIR/go-no-go-gate-fault-report.json"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected go/no-go gate lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_IMPL_SCRIPT" ]; then
  echo "expected go/no-go gate lane implementation script to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi
if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected go/no-go gate lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected go/no-go gate lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected go/no-go gate lane wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_go_no_go_gate_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected go/no-go gate lane manifest to dispatch implementation module" >&2
  exit 1
fi
if ! grep -q 'go_no_go_gate_lane_contract.py' "$LANE_IMPL_SCRIPT"; then
  echo "expected go/no-go gate lane implementation to delegate to contract module" >&2
  exit 1
fi

lane_output="$(
  bash "$LANE_SCRIPT" \
    --max-seconds 120 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected go/no-go gate lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected go/no-go gate lane GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^go_no_go_evidence_status=verified$'; then
  echo "expected go/no-go gate lane evidence status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^rollback_readiness_status=verified$'; then
  echo "expected go/no-go gate lane rollback status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^dr_readiness_status=verified$'; then
  echo "expected go/no-go gate lane dr status marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.go-no-go-gate-report.v1":
    raise SystemExit("unexpected go/no-go gate report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected go/no-go gate report status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected go/no-go gate report final_decision=GO")
if payload.get("fault_profile") != "none":
    raise SystemExit("expected go/no-go gate report fault_profile=none")
if payload.get("go_no_go_evidence_status") != "verified":
    raise SystemExit("expected go_no_go_evidence_status=verified")
if payload.get("rollback_readiness_status") != "verified":
    raise SystemExit("expected rollback_readiness_status=verified")
if payload.get("dr_readiness_status") != "verified":
    raise SystemExit("expected dr_readiness_status=verified")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for baseline go/no-go gate run")
PY

set +e
fault_output="$(
  bash "$LANE_SCRIPT" \
    --fault-profile gate_decision \
    --max-seconds 120 \
    --output-json "$TMP_FAULT_REPORT" 2>&1
)"
fault_code=$?
set -e
if [ "$fault_code" -eq 0 ]; then
  echo "expected go/no-go gate lane gate_decision fault profile to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fault_output" | grep -q 'gate_decision_fault_injection_triggered'; then
  echo "expected go/no-go gate lane gate_decision fault reason marker" >&2
  exit 1
fi

python3 - "$TMP_FAULT_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected gate decision fault report status=fail")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected gate decision fault report final_decision=NO-GO")
if payload.get("fault_profile") != "gate_decision":
    raise SystemExit("expected gate decision fault report fault_profile=gate_decision")
reason_codes = payload.get("reason_codes", [])
if "gate_decision_fault_injection_triggered" not in reason_codes:
    raise SystemExit("expected gate decision fault reason code in report")
PY

set +e
invalid_budget_output="$(
  bash "$LANE_SCRIPT" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected go/no-go gate lane to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_GONOGO_GATE_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for go/no-go gate lane" >&2
  exit 1
fi

echo "go/no-go gate lane script tests passed."
