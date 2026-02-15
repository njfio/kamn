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
    --lane-profile smoke \
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
if ! printf '%s\n' "$validation_output" | grep -q '^lane_profile=smoke$'; then
  echo "expected process-isolated convergence smoke lane profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^smoke_lane_status=verified$'; then
  echo "expected process-isolated convergence smoke lane marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^deep_lane_status=skipped_local_only$'; then
  echo "expected process-isolated convergence deep lane exclusion marker in smoke mode" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_disconnected_fail_closed_status=verified$'; then
  echo "expected process-isolated convergence disconnected fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_disconnected_fail_closed_reason_code=p2p_transport_live_socket_send_failed$'; then
  echo "expected process-isolated convergence disconnected fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_connected_delivery_status=verified$'; then
  echo "expected process-isolated convergence connected delivery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^no_shared_state_zero_delivery_status=verified$'; then
  echo "expected process-isolated convergence no-shared-state zero-delivery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^no_shared_state_unexpected_delivery_reason_code=no_shared_state_unexpected_delivery_detected$'; then
  echo "expected process-isolated convergence no-shared-state unexpected-delivery reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^no_shared_state_delivery_count=0$'; then
  echo "expected process-isolated convergence no-shared-state delivery-count marker" >&2
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
if ! printf '%s\n' "$validation_output" | grep -q '^native_compile_mode_status=verified$'; then
  echo "expected process-isolated convergence native compile-mode marker" >&2
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
if payload.get("lane_profile") != "smoke":
    raise SystemExit("expected lane_profile=smoke")
if payload.get("smoke_lane_status") != "verified":
    raise SystemExit("expected smoke_lane_status=verified")
if payload.get("deep_lane_status") != "skipped_local_only":
    raise SystemExit("expected deep_lane_status=skipped_local_only for smoke lane")
if payload.get("two_node_disconnected_fail_closed_status") != "verified":
    raise SystemExit("expected disconnected fail-closed status marker")
if payload.get("two_node_disconnected_fail_closed_reason_code") != "p2p_transport_live_socket_send_failed":
    raise SystemExit("expected disconnected fail-closed reason code marker")
if payload.get("two_node_connected_delivery_status") != "verified":
    raise SystemExit("expected connected delivery status marker")
if payload.get("no_shared_state_zero_delivery_status") != "verified":
    raise SystemExit("expected no-shared-state zero-delivery status marker")
if payload.get("no_shared_state_unexpected_delivery_reason_code") != "no_shared_state_unexpected_delivery_detected":
    raise SystemExit("expected no-shared-state unexpected-delivery reason marker")
if payload.get("no_shared_state_delivery_count") != 0:
    raise SystemExit("expected no-shared-state delivery-count marker")
if payload.get("native_compile_mode_status") != "verified":
    raise SystemExit("expected native compile-mode marker")
PY

smoke_run_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --lane-profile smoke \
    --max-seconds 180 \
    --command-max-seconds 120 \
    --ci-fast-gate PASS
)"
if ! printf '%s\n' "$smoke_run_output" | grep -q '^execution_reason_code=run_mode_smoke_commands_executed$'; then
  echo "expected smoke run-mode command execution marker" >&2
  exit 1
fi
if ! printf '%s\n' "$smoke_run_output" | grep -q '^command_count=3$'; then
  echo "expected smoke run-mode command_count=3 marker" >&2
  exit 1
fi

set +e
run_without_opt_in_output="$({
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --lane-profile deep \
    --max-seconds 120 \
    --ci-fast-gate FAIL
} 2>&1)"
run_without_opt_in_code=$?
set -e
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected deep run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_DEEP_OPT_IN=1'; then
  echo "expected deterministic deep-lane opt-in marker for process-isolated convergence run mode" >&2
  exit 1
fi

echo "process-isolated libp2p convergence validation tests passed."
