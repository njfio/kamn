#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_three_node_discovery_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected libp2p three-node discovery live validation script to be executable" >&2
  exit 1
fi

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected libp2p three-node discovery lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected libp2p three-node discovery lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected libp2p three-node discovery lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_discovery_status=verified$'; then
  echo "expected libp2p three-node discovery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^gossip_propagation_status=verified$'; then
  echo "expected libp2p gossip propagation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lifecycle_transition_status=verified$'; then
  echo "expected libp2p lifecycle transition marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_discovery_gossip_three_node$'; then
  echo "expected libp2p runtime transport mode marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-three-node-discovery-live-report.v1":
    raise SystemExit("unexpected libp2p three-node discovery report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("execution_reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
if payload.get("command_count") != 0:
    raise SystemExit("expected command_count=0 for dry-run mode")
if payload.get("three_node_discovery_status") != "verified":
    raise SystemExit("expected three_node_discovery_status=verified")
if payload.get("gossip_propagation_status") != "verified":
    raise SystemExit("expected gossip_propagation_status=verified")
if payload.get("lifecycle_transition_status") != "verified":
    raise SystemExit("expected lifecycle_transition_status=verified")
if payload.get("runtime_transport_mode") != "libp2p_discovery_gossip_three_node":
    raise SystemExit("expected deterministic runtime transport mode marker")
PY

set +e
invalid_mode_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode invalid 2>&1
)"
invalid_mode_code=$?
set -e
if [ "$invalid_mode_code" -eq 0 ]; then
  echo "expected libp2p three-node discovery lane to reject invalid mode" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_mode_output" | grep -q -- '--mode must be one of: dry-run, run'; then
  echo "expected deterministic invalid mode marker" >&2
  exit 1
fi

echo "libp2p three-node discovery live validation tests passed."
