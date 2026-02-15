#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected sqlite crash-recovery live validation script to be executable" >&2
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
  echo "expected sqlite crash-recovery live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected sqlite crash-recovery live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected sqlite crash-recovery live validation dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fast_gate_exclusion_status=verified$'; then
  echo "expected sqlite crash-recovery live validation fast-gate exclusion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^sqlite_crash_recovery_state_replay_status=verified$'; then
  echo "expected sqlite crash-recovery live validation replay marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^sqlite_crash_recovery_abrupt_kill_status=verified$'; then
  echo "expected sqlite crash-recovery live validation abrupt-kill marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_append_status=verified$'; then
  echo "expected sqlite crash-recovery live validation wal-append marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_checkpoint_status=verified$'; then
  echo "expected sqlite crash-recovery live validation wal-checkpoint marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_durability_reason_taxonomy_version=kamn.runtime.wal-durability-reason-taxonomy.v1$'; then
  echo "expected sqlite crash-recovery live validation wal-durability reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^wal_durability_reason_codes_csv=wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete$'; then
  echo "expected sqlite crash-recovery live validation wal-durability reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^run_mode_command_status=dry_run_no_commands_executed$'; then
  echo "expected sqlite crash-recovery live validation dry-run command marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.sqlite-crash-recovery-live-report.v1":
    raise SystemExit("unexpected sqlite crash-recovery live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected sqlite crash-recovery live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected sqlite crash-recovery live validation final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("ci_fast_gate_eligibility") != "eligible":
    raise SystemExit("expected ci_fast_gate_eligibility=eligible")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 for dry-run")
if payload.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
if payload.get("wal_append_status") != "verified":
    raise SystemExit("expected wal_append_status=verified")
if payload.get("wal_checkpoint_status") != "verified":
    raise SystemExit("expected wal_checkpoint_status=verified")
if payload.get("wal_durability_reason_taxonomy_version") != "kamn.runtime.wal-durability-reason-taxonomy.v1":
    raise SystemExit("expected deterministic wal_durability_reason_taxonomy_version marker")
if payload.get("wal_durability_reason_codes_csv") != "wal_append_rejected,wal_checkpoint_skipped,wal_replay_incomplete":
    raise SystemExit("expected deterministic wal_durability_reason_codes_csv marker")
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
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_SQLITE_CRASH_RECOVERY_LIVE_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for sqlite crash-recovery run mode" >&2
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
  echo "expected sqlite crash-recovery validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_SQLITE_CRASH_RECOVERY_LIVE_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for sqlite crash-recovery validation script" >&2
  exit 1
fi

echo "sqlite crash-recovery live validation tests passed."
