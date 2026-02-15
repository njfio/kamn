#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh"
TMP_REPORT="$(mktemp)"
TMP_RUN_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_RUN_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected block reconciliation partition/rejoin live validation script to be executable" >&2
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
  echo "expected block reconciliation partition/rejoin validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected block reconciliation partition/rejoin validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected block reconciliation partition/rejoin validation dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected block reconciliation partition/rejoin validation fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^block_reconciliation_partition_status=verified$'; then
  echo "expected block reconciliation partition marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^block_reconciliation_rejoin_status=verified$'; then
  echo "expected block reconciliation rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^canonical_convergence_status=verified$'; then
  echo "expected canonical convergence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_transport_fed$'; then
  echo "expected runtime transport mode marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconciliation_reason_taxonomy_status=verified$'; then
  echo "expected reconciliation reason taxonomy status marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconciliation_reason_codes=none$'; then
  echo "expected deterministic reconciliation reason-code matrix marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^run_mode_command_status=dry_run_no_commands_executed$'; then
  echo "expected block reconciliation partition/rejoin dry-run command marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected block reconciliation partition/rejoin status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected block reconciliation partition/rejoin final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("ci_fast_gate_eligibility") != "eligible":
    raise SystemExit("expected ci_fast_gate_eligibility=eligible")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 for dry-run")
if payload.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
if payload.get("runtime_transport_mode") != "libp2p_transport_fed":
    raise SystemExit("expected runtime_transport_mode=libp2p_transport_fed")
if payload.get("transport_state_transition_status") != "verified":
    raise SystemExit("expected transport_state_transition_status=verified")
if payload.get("reconciliation_reason_taxonomy_status") != "verified":
    raise SystemExit("expected reconciliation_reason_taxonomy_status=verified")
if payload.get("reconciliation_reason_taxonomy_version") != "kamn.runtime.block-reconciliation-partition-rejoin-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reconciliation reason taxonomy version")
if payload.get("reconciliation_reason_codes") != ["none"]:
    raise SystemExit("expected deterministic reconciliation_reason_codes=['none'] for dry-run")
PY

python3 - "$ROOT_DIR" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1]).resolve()
sys.path.insert(0, str(root / "scripts" / "runtime"))
import block_reconciliation_partition_rejoin_live_contract as contract

synthetic_report = {
    "scenario_results": [
        {"scenario": "primary_loss_reconnect_catchup", "status": "fail"},
        {"scenario": "reconnect_drift_regression", "status": "fail"},
    ],
    "reason_codes": ["runtime_budget_exceeded", "ci_fast_gate_failed"],
}
codes = contract._derive_reconciliation_reason_codes(synthetic_report, lane_mode="run")
expected = [
    "reconciliation_ci_fast_gate_failed",
    "reconciliation_partition_transition_failed",
    "reconciliation_rejoin_transition_failed",
    "reconciliation_runtime_budget_exceeded",
]
if codes != expected:
    raise SystemExit(f"unexpected reconciliation taxonomy codes: expected={expected}, found={codes}")
if contract._derive_reconciliation_reason_codes(None, lane_mode="dry-run") != ["none"]:
    raise SystemExit("expected dry-run taxonomy derivation to return ['none']")
PY

run_output="$(
  KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_OPT_IN=1 \
    bash "$VALIDATION_SCRIPT" \
      --mode run \
      --max-seconds 120 \
      --ci-fast-gate PASS \
      --output-json "$TMP_RUN_REPORT"
)"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected block reconciliation partition/rejoin run-mode validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^lane_mode=run$'; then
  echo "expected block reconciliation partition/rejoin run-mode lane marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^runtime_transport_mode=libp2p_transport_fed$'; then
  echo "expected run-mode runtime transport mode marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^reconciliation_reason_codes=none$'; then
  echo "expected run-mode deterministic reconciliation reason-code matrix marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^run_mode_command_status=executed$'; then
  echo "expected block reconciliation partition/rejoin run-mode command execution marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^reason_code=block_reconciliation_partition_rejoin_live_validation_executed$'; then
  echo "expected deterministic run-mode reason code marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi

python3 - "$TMP_RUN_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("lane_mode") != "run":
    raise SystemExit("expected lane_mode=run")
if payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy":
    raise SystemExit("expected ci_fast_gate_eligibility=excluded_local_heavy")
if payload.get("run_mode_command_status") != "executed":
    raise SystemExit("expected run_mode_command_status=executed")
if payload.get("run_mode_command_count", 0) <= 0:
    raise SystemExit("expected run_mode_command_count>0 for run mode")
if payload.get("runtime_transport_mode") != "libp2p_transport_fed":
    raise SystemExit("expected runtime_transport_mode=libp2p_transport_fed")
if payload.get("reconciliation_reason_codes") != ["none"]:
    raise SystemExit("expected deterministic reconciliation_reason_codes=['none'] for run mode")
PY

set +e
run_without_opt_in_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --max-seconds 120 \
    --ci-fast-gate PASS 2>&1
)"
run_without_opt_in_code=$?
set -e
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected block reconciliation partition/rejoin run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for block reconciliation partition/rejoin run mode" >&2
  exit 1
fi

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected block reconciliation partition/rejoin validation to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_BLOCK_RECONCILIATION_PARTITION_REJOIN_LIVE_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for block reconciliation partition/rejoin validation" >&2
  exit 1
fi

echo "block reconciliation partition/rejoin live validation tests passed."
