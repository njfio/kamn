#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local retry/diagnostics live validation script to be executable" >&2
  exit 1
fi

nonce_stub="$TMP_DIR/nonce-stub.sh"
cat >"$nonce_stub" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "status=pass"
echo "final_decision=GO"
echo "nonce_retry_contract_status=verified"
SH
chmod +x "$nonce_stub"

structured_stub="$TMP_DIR/structured-stub.sh"
cat >"$structured_stub" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
echo "status=pass"
echo "final_decision=GO"
echo "correlation_contract_status=verified"
SH
chmod +x "$structured_stub"

dry_run_report="$TMP_DIR/local-retry-diagnostics-dry-run.json"
dry_run_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 60 \
    --output-json "$dry_run_report"
)"
if ! printf '%s\n' "$dry_run_output" | grep -q '^status=pass$'; then
  echo "expected local retry/diagnostics dry-run status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local retry/diagnostics dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^execution_reason_code=dry_run_no_commands_executed$'; then
  echo "expected local retry/diagnostics dry-run reason marker" >&2
  exit 1
fi

python3 - "$dry_run_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-report.v1":
    raise SystemExit("unexpected local retry/diagnostics dry-run schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local retry/diagnostics dry-run status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local retry/diagnostics dry-run final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected local retry/diagnostics dry-run lane_mode=dry-run")
if payload.get("command_count") != 0:
    raise SystemExit("expected local retry/diagnostics dry-run command_count=0")
if payload.get("execution_reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected local retry/diagnostics dry-run reason code")
PY

set +e
missing_opt_in_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --max-seconds 60 \
    --nonce-retry-script "$nonce_stub" \
    --structured-logging-script "$structured_stub" 2>&1
)"
missing_opt_in_code=$?
set -e
if [ "$missing_opt_in_code" -eq 0 ]; then
  echo "expected local retry/diagnostics run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_LOCAL_RETRY_DIAGNOSTICS_OPT_IN=1'; then
  echo "expected deterministic opt-in failure marker for local retry/diagnostics run mode" >&2
  exit 1
fi

run_report="$TMP_DIR/local-retry-diagnostics-run.json"
run_output="$(
  KAMN_LOCAL_RETRY_DIAGNOSTICS_OPT_IN=1 \
    bash "$VALIDATION_SCRIPT" \
      --mode run \
      --max-seconds 60 \
      --command-max-seconds 10 \
      --nonce-retry-script "$nonce_stub" \
      --structured-logging-script "$structured_stub" \
      --output-json "$run_report"
)"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected local retry/diagnostics run status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^lane_mode=run$'; then
  echo "expected local retry/diagnostics run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^execution_reason_code=run_mode_commands_executed$'; then
  echo "expected local retry/diagnostics run reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^command_count=2$'; then
  echo "expected local retry/diagnostics run command-count marker" >&2
  exit 1
fi

python3 - "$run_report" "$nonce_stub" "$structured_stub" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
expected_commands = [str(pathlib.Path(sys.argv[2]).resolve()), str(pathlib.Path(sys.argv[3]).resolve())]
if payload.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-report.v1":
    raise SystemExit("unexpected local retry/diagnostics run schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local retry/diagnostics run status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local retry/diagnostics run final_decision=GO")
if payload.get("lane_mode") != "run":
    raise SystemExit("expected local retry/diagnostics run lane_mode=run")
if payload.get("command_count") != 2:
    raise SystemExit("expected local retry/diagnostics run command_count=2")
if payload.get("commands") != expected_commands:
    raise SystemExit("unexpected local retry/diagnostics run command list")
if payload.get("execution_reason_code") != "run_mode_commands_executed":
    raise SystemExit("expected local retry/diagnostics run reason code")
if payload.get("retry_contract_status") != "verified":
    raise SystemExit("expected local retry/diagnostics retry marker")
if payload.get("correlation_diagnostics_status") != "verified":
    raise SystemExit("expected local retry/diagnostics correlation marker")
PY

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds invalid 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected local retry/diagnostics validation to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_LOCAL_RETRY_DIAGNOSTICS_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for local retry/diagnostics validation" >&2
  exit 1
fi

echo "local retry/diagnostics live validation tests passed."
