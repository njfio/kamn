#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected live transport fault matrix validation script to be executable" >&2
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
  echo "expected live transport fault matrix status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected live transport fault matrix final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^partition_rejoin_status=verified$'; then
  echo "expected live transport fault matrix partition/rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^publish_drop_recovery_status=verified$'; then
  echo "expected live transport fault matrix publish-drop marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^replay_recovery_status=verified$'; then
  echo "expected live transport fault matrix replay marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_churn_recovery_status=verified$'; then
  echo "expected live transport fault matrix churn marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_live_fault_matrix$'; then
  echo "expected live transport fault matrix runtime transport marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1$'; then
  echo "expected live transport fault matrix peer-adapter reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer$'; then
  echo "expected live transport fault matrix peer-integrity fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout$'; then
  echo "expected live transport fault matrix retry-timeout reason projection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted$'; then
  echo "expected live transport fault matrix retry-budget-exhausted reason projection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^peer_adapter_multi_process_validation_local_heavy_status=required$'; then
  echo "expected live transport fault matrix peer-adapter multi-process local-heavy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reason_taxonomy_version=kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1$'; then
  echo "expected live transport fault matrix reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected live transport fault matrix normalized reason codes value marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-transport-fault-matrix-report.v1":
    raise SystemExit("unexpected live transport fault matrix report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("runtime_transport_mode") != "libp2p_live_fault_matrix":
    raise SystemExit("expected runtime transport mode marker")
if payload.get("peer_adapter_reason_taxonomy_version") != "kamn.runtime.peer-adapter-reason-taxonomy.v1":
    raise SystemExit("expected deterministic peer_adapter_reason_taxonomy_version marker")
if payload.get("peer_integrity_fail_closed_reason_code") != "p2p_transport_unknown_sender_peer":
    raise SystemExit("expected deterministic peer_integrity_fail_closed_reason_code marker")
if payload.get("peer_adapter_reason_projection_timeout_code") != "p2p_live_reconnect_retry_dial_timeout":
    raise SystemExit("expected deterministic peer_adapter_reason_projection_timeout_code marker")
if payload.get("peer_adapter_reason_projection_budget_exhausted_code") != "p2p_live_reconnect_retry_budget_exhausted":
    raise SystemExit("expected deterministic peer_adapter_reason_projection_budget_exhausted_code marker")
if payload.get("peer_adapter_multi_process_validation_local_heavy_status") != "required":
    raise SystemExit("expected deterministic peer_adapter_multi_process_validation_local_heavy_status marker")
if payload.get("reason_taxonomy_version") != "kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy version marker")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected deterministic reason_codes=['none']")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none")
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
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'KAMN_LIVE_TRANSPORT_FAULT_MATRIX_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for live transport fault matrix run mode" >&2
  exit 1
fi

echo "live transport fault matrix validation tests passed."
