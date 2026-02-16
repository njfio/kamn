#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_signal_secret_hygiene_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local signal/secret hygiene validation script to be executable" >&2
  exit 1
fi

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 240 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local signal/secret hygiene validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local signal/secret hygiene validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local signal/secret hygiene validation dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^signal_shutdown_status=verified$'; then
  echo "expected local signal/secret hygiene validation signal marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^signal_graceful_drain_status=verified$'; then
  echo "expected local signal/secret hygiene validation graceful-drain marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^shutdown_reason_taxonomy_version=kamn.runtime.local-signal-shutdown-reason-taxonomy.v1$'; then
  echo "expected local signal/secret hygiene validation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^shutdown_reason_codes_csv=local_signal_shutdown_path_drift_detected,local_graceful_drain_bypass_detected,ci_local_signal_shutdown_budget_boundary_exceeded$'; then
  echo "expected local signal/secret hygiene validation reason codes taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^secret_hygiene_status=verified$'; then
  echo "expected local signal/secret hygiene validation secret-hygiene marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^secret_hygiene_policy_status=verified$'; then
  echo "expected local signal/secret hygiene validation policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fallback_secret_fail_closed_reason_code=fallback_signer_secret_present_violation$'; then
  echo "expected local signal/secret hygiene validation fail-closed reason marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-signal-secret-hygiene-live-report.v1":
    raise SystemExit("unexpected local signal/secret hygiene validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local signal/secret hygiene validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local signal/secret hygiene validation final_decision=GO")
if payload.get("signal_shutdown_status") != "verified":
    raise SystemExit("expected signal_shutdown_status=verified")
if payload.get("signal_graceful_drain_status") != "verified":
    raise SystemExit("expected signal_graceful_drain_status=verified")
if payload.get("shutdown_reason_taxonomy_version") != "kamn.runtime.local-signal-shutdown-reason-taxonomy.v1":
    raise SystemExit("expected deterministic shutdown reason taxonomy marker")
if payload.get("shutdown_reason_codes_csv") != "local_signal_shutdown_path_drift_detected,local_graceful_drain_bypass_detected,ci_local_signal_shutdown_budget_boundary_exceeded":
    raise SystemExit("expected deterministic shutdown reason codes taxonomy marker")
if payload.get("secret_hygiene_status") != "verified":
    raise SystemExit("expected secret_hygiene_status=verified")
if payload.get("secret_hygiene_policy_status") != "verified":
    raise SystemExit("expected secret_hygiene_policy_status=verified")
if payload.get("fallback_secret_fail_closed_reason_code") != "fallback_signer_secret_present_violation":
    raise SystemExit("expected deterministic fallback secret fail-closed reason code marker")
if payload.get("ci_local_signal_budget_boundary_status") != "verified":
    raise SystemExit("expected ci_local_signal_budget_boundary_status=verified")
if payload.get("max_seconds") != 240:
    raise SystemExit("expected max_seconds=240")
PY

echo "local signal/secret hygiene live validation tests passed."
