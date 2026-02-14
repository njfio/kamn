#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/deploy/validate_local_compose_multinode_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local compose multinode live validation script to be executable" >&2
  exit 1
fi

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local compose multinode live validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local compose multinode live validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local compose multinode live validation lane_mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^compose_startup_status=verified$'; then
  echo "expected local compose multinode live validation startup marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^compose_health_status=verified$'; then
  echo "expected local compose multinode live validation health marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^compose_shutdown_status=verified$'; then
  echo "expected local compose multinode live validation shutdown marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^run_mode_command_status=dry_run_no_commands_executed$'; then
  echo "expected local compose multinode dry-run command marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.deploy.local-compose-multinode-live-report.v1":
    raise SystemExit("unexpected local compose multinode live schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local compose multinode status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local compose multinode final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 in dry-run")
if payload.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
PY

set +e
missing_opt_in_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --ci-fast-gate PASS 2>&1
)"
missing_opt_in_code=$?
set -e
if [ "$missing_opt_in_code" -eq 0 ]; then
  echo "expected local compose multinode run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_LOCAL_COMPOSE_MULTINODE_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for local compose multinode run mode" >&2
  exit 1
fi

echo "local compose multinode live validation tests passed."
