#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/validate_compose_topology_contract_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected compose topology contract lane script to be executable" >&2
  exit 1
fi

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$TMP_REPORT" \
    --max-seconds 240 \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected compose topology contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected compose topology contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^compose_runtime_mode_full_status=verified$'; then
  echo "expected compose topology contract lane runtime-mode full marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^compose_api_port_status=verified$'; then
  echo "expected compose topology contract lane api-port marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^compose_volume_network_status=verified$'; then
  echo "expected compose topology contract lane volume/network marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^compose_docs_parity_status=verified$'; then
  echo "expected compose topology contract lane docs marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.deploy.compose-topology-contract-lane-summary.v1":
    raise SystemExit("unexpected compose topology contract lane schema")
if payload.get("status") != "pass":
    raise SystemExit("expected compose topology contract lane status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected compose topology contract lane final_decision=GO")
if payload.get("compose_runtime_mode_full_status") != "verified":
    raise SystemExit("expected compose_runtime_mode_full_status=verified")
if payload.get("compose_api_port_status") != "verified":
    raise SystemExit("expected compose_api_port_status=verified")
if payload.get("compose_volume_network_status") != "verified":
    raise SystemExit("expected compose_volume_network_status=verified")
if payload.get("compose_docs_parity_status") != "verified":
    raise SystemExit("expected compose_docs_parity_status=verified")
PY

set +e
invalid_budget_output="$(
  bash "$CONTRACT_LANE" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected compose topology contract lane to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for compose topology contract lane" >&2
  exit 1
fi

echo "compose topology contract lane tests passed."
