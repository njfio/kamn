#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_libp2p_process_isolated_harness_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_process_isolated_harness.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_process_isolated_harness_policy.sh"

for required_exec in "$CONTRACT_LANE" "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected process-isolated harness script to be executable: $required_exec" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/libp2p-process-isolated-harness-contract-lane-report.json"
policy_report="$TMP_DIR/libp2p-process-isolated-harness-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 180 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated harness contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated harness contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected process-isolated harness contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_process_isolated_harness_contract_status=verified$'; then
  echo "expected process-isolated harness contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_process_isolated_harness_policy_status=verified$'; then
  echo "expected process-isolated harness policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=libp2p_process_isolated_harness_policy_marker_missing:partition_rejoin_status$'; then
  echo "expected process-isolated harness fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if lane_payload.get("schema_version") != "kamn.runtime.libp2p-process-isolated-harness-contract-lane-report.v1":
    raise SystemExit("unexpected process-isolated harness contract-lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected process-isolated harness contract-lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated harness contract-lane final_decision=GO")
if lane_payload.get("libp2p_process_isolated_harness_contract_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_harness_contract_status=verified")
if lane_payload.get("libp2p_process_isolated_harness_policy_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_harness_policy_status=verified")

if policy_payload.get("schema_version") != "kamn.runtime.libp2p-process-isolated-harness-policy-report.v1":
    raise SystemExit("unexpected process-isolated harness policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated harness policy final_decision=GO")
if policy_payload.get("libp2p_process_isolated_harness_policy_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_harness_policy_status=verified in policy report")
PY

set +e
invalid_mode_output="$(
  bash "$CONTRACT_LANE" \
    --mode maybe \
    --max-seconds 120 2>&1
)"
invalid_mode_code=$?
set -e
if [ "$invalid_mode_code" -eq 0 ]; then
  echo "expected process-isolated harness contract lane to reject invalid mode" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_mode_output" | grep -q 'mode must be dry-run or run'; then
  echo "expected deterministic invalid mode marker for process-isolated harness contract lane" >&2
  exit 1
fi

echo "process-isolated libp2p harness contract lane tests passed."
