#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected process-isolated convergence validation script to be executable" >&2
  exit 1
fi

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 120 \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated convergence status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated convergence final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_discovery_status=verified$'; then
  echo "expected process-isolated convergence two-node discovery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_gossip_status=verified$'; then
  echo "expected process-isolated convergence two-node gossip marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_partition_rejoin_status=verified$'; then
  echo "expected process-isolated convergence partition/rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_publish_drop_recovery_status=verified$'; then
  echo "expected process-isolated convergence publish-drop marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^convergence_reason_code_status=verified$'; then
  echo "expected process-isolated convergence reason-code marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_process_isolated_convergence$'; then
  echo "expected process-isolated convergence runtime transport marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-report.v1":
    raise SystemExit("unexpected process-isolated convergence report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("runtime_transport_mode") != "libp2p_process_isolated_convergence":
    raise SystemExit("expected runtime transport mode marker")
reason_codes = payload.get("convergence_reason_codes")
if reason_codes != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected deterministic convergence reason-code marker")
PY

set +e
run_without_opt_in_output="$({
  bash "$VALIDATION_SCRIPT" --mode run --max-seconds 120 --ci-fast-gate PASS
} 2>&1)"
run_without_opt_in_code=$?
set -e
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_LIVE_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for process-isolated convergence run mode" >&2
  exit 1
fi

echo "process-isolated libp2p convergence validation tests passed."
