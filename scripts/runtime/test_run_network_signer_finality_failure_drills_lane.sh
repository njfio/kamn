#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/runtime/run_network_signer_finality_failure_drills_lane.sh"
LANE_IMPL_SCRIPT="$ROOT_DIR/scripts/runtime/run_network_signer_finality_failure_drills_lane_impl.sh"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/runtime_network_signer_finality_failure_drills_lane.json"
TMP_DIR="$(mktemp -d)"
TMP_REPORT="$TMP_DIR/failure-drills-report.json"
TMP_FAULT_REPORT="$TMP_DIR/failure-drills-fault-report.json"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected network/signer/finality failure drills lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$LANE_IMPL_SCRIPT" ]; then
  echo "expected network/signer/finality failure drills lane implementation script to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected shared non-Kolme dispatcher to be executable" >&2
  exit 1
fi

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected network/signer/finality failure drills lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected network/signer/finality failure drills lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected network/signer/finality failure drills lane wrapper to resolve runtime manifest via dispatcher" >&2
  exit 1
fi

if ! grep -q 'run_network_signer_finality_failure_drills_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected network/signer/finality failure drills lane manifest to dispatch implementation module" >&2
  exit 1
fi
if ! grep -q 'network_signer_finality_failure_drills_lane_contract.py' "$LANE_IMPL_SCRIPT"; then
  echo "expected network/signer/finality failure drills lane implementation to delegate to contract module" >&2
  exit 1
fi

lane_output="$(
  bash "$LANE_SCRIPT" \
    --max-seconds 180 \
    --partition-max-seconds 60 \
    --signer-max-seconds 60 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected failure drills lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected failure drills lane GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^network_partition_status=verified$'; then
  echo "expected failure drills lane network partition marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^signer_fault_status=verified$'; then
  echo "expected failure drills lane signer marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^finality_fault_status=verified$'; then
  echo "expected failure drills lane finality marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.failure-drills-report.v1":
    raise SystemExit("unexpected failure drills report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected failure drills report status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected failure drills report final_decision=GO")
if payload.get("fault_profile") != "none":
    raise SystemExit("expected failure drills report fault_profile=none")
if payload.get("network_partition_status") != "verified":
    raise SystemExit("expected network_partition_status=verified")
if payload.get("signer_fault_status") != "verified":
    raise SystemExit("expected signer_fault_status=verified")
if payload.get("finality_fault_status") != "verified":
    raise SystemExit("expected finality_fault_status=verified")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for baseline drill")
PY

set +e
fault_output="$(
  bash "$LANE_SCRIPT" \
    --fault-profile signer \
    --max-seconds 180 \
    --partition-max-seconds 60 \
    --signer-max-seconds 60 \
    --output-json "$TMP_FAULT_REPORT" 2>&1
)"
fault_code=$?
set -e
if [ "$fault_code" -eq 0 ]; then
  echo "expected signer fault-profile run to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fault_output" | grep -q "signer_fault_injection_triggered"; then
  echo "expected signer fault-injection reason code marker" >&2
  exit 1
fi

python3 - "$TMP_FAULT_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected signer fault report status=fail")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected signer fault report final_decision=NO-GO")
if payload.get("fault_profile") != "signer":
    raise SystemExit("expected signer fault report fault_profile=signer")
reason_codes = payload.get("reason_codes", [])
if "signer_fault_injection_triggered" not in reason_codes:
    raise SystemExit("expected signer fault reason code in report")
PY

set +e
invalid_budget_output="$(
  bash "$LANE_SCRIPT" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected failure drills lane to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_FAILURE_DRILLS_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for failure drills lane" >&2
  exit 1
fi

echo "network/signer/finality failure drills lane script tests passed."
