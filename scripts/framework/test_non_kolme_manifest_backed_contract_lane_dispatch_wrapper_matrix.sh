#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
DELETION_MANIFEST="$ROOT_DIR/fixtures/ci/superseded_script_deletion_manifest.json"

is_superseded_deleted_wrapper() {
  local wrapper_rel="$1"
  python3 - "$DELETION_MANIFEST" "$wrapper_rel" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
wrapper_rel = sys.argv[2]
if not manifest_path.is_file():
    raise SystemExit(1)
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
for entry in payload.get("deletions", []):
    if isinstance(entry, dict) and entry.get("script_path") == wrapper_rel:
        raise SystemExit(0)
raise SystemExit(1)
PY
}

if [ ! -x "$DISPATCHER" ]; then
  echo "expected non-Kolme contract-lane dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

lane_wrappers=(
  "scripts/canary/run_launch_canary_contract_lane.sh"
  "scripts/canary/run_post_cutover_slo_contract_lane.sh"
  "scripts/dashboard/run_backend_session_auth_freshness_contract_lane.sh"
  "scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh"
  "scripts/guard/run_durable_guard_recovery_contract_lane.sh"
  "scripts/reputation/run_reputation_dispute_contract_lane.sh"
  "scripts/token/run_token_launch_handoff_contract_lane.sh"
  "scripts/treasury/run_treasury_disbursement_contract_lane.sh"
)

for wrapper_rel_path in "${lane_wrappers[@]}"; do
  wrapper_path="$ROOT_DIR/$wrapper_rel_path"
  wrapper_name="$(basename "$wrapper_path")"

  if [ -e "$wrapper_path" ]; then
    if [ ! -x "$wrapper_path" ]; then
      echo "expected wrapper to be executable: $wrapper_path" >&2
      exit 1
    fi

    if [ ! -L "$wrapper_path" ]; then
      echo "expected wrapper to be a symlink to shared dispatcher: $wrapper_path" >&2
      exit 1
    fi
  elif ! is_superseded_deleted_wrapper "$wrapper_rel_path"; then
    echo "expected wrapper to exist or be cataloged as superseded/deleted: $wrapper_path" >&2
    exit 1
  fi

  manifest_path="$(bash "$DISPATCHER" --lane-wrapper "$wrapper_name" --resolve-manifest-path)"
  if [ ! -f "$manifest_path" ]; then
    echo "expected dispatcher to resolve existing manifest for $wrapper_name: $manifest_path" >&2
    exit 1
  fi

  python3 - "$manifest_path" "$wrapper_name" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
wrapper_name = sys.argv[2]
payload = json.loads(manifest_path.read_text(encoding="utf-8"))

if payload.get("wrapper_name") != wrapper_name:
    print(
        f"expected wrapper_name={wrapper_name!r} in {manifest_path.name}, got {payload.get('wrapper_name')!r}",
        file=sys.stderr,
    )
    raise SystemExit(1)

phase = payload.get("phase")
if not isinstance(phase, str) or phase.strip() == "":
    print(f"expected non-empty phase field in {manifest_path.name}", file=sys.stderr)
    raise SystemExit(1)

phases = payload.get("phases")
if not isinstance(phases, dict) or phase not in phases:
    print(
        f"expected phase {phase!r} to exist in phases for {manifest_path.name}",
        file=sys.stderr,
    )
    raise SystemExit(1)
PY
done

if bash "$DISPATCHER" --lane-wrapper run_missing_non_kolme_manifest_wrapper_contract_lane.sh --resolve-manifest-path >/dev/null 2>&1; then
  echo "expected non-Kolme dispatcher to fail for unknown manifest-backed wrapper" >&2
  exit 1
fi

echo "non-Kolme manifest-backed contract lane dispatcher wrapper matrix tests passed."
