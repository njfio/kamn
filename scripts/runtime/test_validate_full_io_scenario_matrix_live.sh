#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_full_io_scenario_matrix_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected full I/O scenario matrix validation script to be executable" >&2
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
  echo "expected full I/O scenario matrix validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected full I/O scenario matrix validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected full I/O scenario matrix validation dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^process_harness_contract_status=verified$'; then
  echo "expected full I/O scenario matrix process harness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^api_route_matrix_status=verified$'; then
  echo "expected full I/O scenario matrix API marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^auth_failure_matrix_status=verified$'; then
  echo "expected full I/O scenario matrix auth marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_matrix_status=verified$'; then
  echo "expected full I/O scenario matrix websocket marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^multinode_propagation_status=verified$'; then
  echo "expected full I/O scenario matrix multinode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^run_mode_command_status=dry_run_no_commands_executed$'; then
  echo "expected full I/O scenario matrix dry-run command status marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.full-io-scenario-matrix-live-report.v1":
    raise SystemExit("unexpected full I/O scenario matrix validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected full I/O scenario matrix validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected full I/O scenario matrix validation final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("ci_fast_gate_eligibility") != "eligible":
    raise SystemExit("expected ci_fast_gate_eligibility=eligible")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 for dry-run")
if payload.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
if not isinstance(payload.get("scenario_artifact_paths"), dict):
    raise SystemExit("expected scenario_artifact_paths dictionary")
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
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_LOCAL_FULL_IO_SCENARIO_MATRIX_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for full I/O scenario matrix run mode" >&2
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
  echo "expected full I/O scenario matrix validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_FULL_IO_SCENARIO_MATRIX_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for full I/O scenario matrix validation script" >&2
  exit 1
fi

echo "full I/O scenario matrix live validation tests passed."
