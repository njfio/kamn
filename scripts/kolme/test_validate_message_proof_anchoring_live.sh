#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/kolme/validate_message_proof_anchoring_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected message proof anchoring live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected message proof anchoring live pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected message proof anchoring live GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^message_anchor_contract_status=verified$'; then
  echo "expected message anchor contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected evidence bundle marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^docs_contract_status=verified$'; then
  echo "expected docs contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_reason_code=message_proof_anchor_conflicting_key$'; then
  echo "expected fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1$'; then
  echo "expected deterministic anchoring gate reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required$'; then
  echo "expected deterministic anchoring gate reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^ci_smoke_local_heavy_boundary_status=verified$'; then
  echo "expected ci smoke/local-heavy boundary marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^ci_smoke_lane_cost_profile=low$'; then
  echo "expected ci smoke lane cost profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^local_heavy_lane_execution_mode=opt_in$'; then
  echo "expected local-heavy lane execution mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^performance_budget_status=verified$'; then
  echo "expected performance marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.message-proof-anchoring.live-validation.v1":
    raise SystemExit("unexpected message proof anchoring live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("message_anchor_contract_status") != "verified":
    raise SystemExit("expected message_anchor_contract_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("fail_closed_reason_code") != "message_proof_anchor_conflicting_key":
    raise SystemExit("expected fail_closed_reason_code=message_proof_anchor_conflicting_key")
if payload.get("anchoring_gate_reason_taxonomy_version") != "kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1":
    raise SystemExit("expected anchoring_gate_reason_taxonomy_version")
if payload.get("anchoring_gate_reason_codes_csv") != "message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required":
    raise SystemExit("expected deterministic anchoring_gate_reason_codes_csv")
if payload.get("anchoring_gate_reason_codes_value") != "message_proof_anchor_conflicting_key":
    raise SystemExit("expected anchoring_gate_reason_codes_value=message_proof_anchor_conflicting_key")
if payload.get("ci_smoke_local_heavy_boundary_status") != "verified":
    raise SystemExit("expected ci_smoke_local_heavy_boundary_status=verified")
if payload.get("ci_smoke_lane_cost_profile") != "low":
    raise SystemExit("expected ci_smoke_lane_cost_profile=low")
if payload.get("local_heavy_lane_execution_mode") != "opt_in":
    raise SystemExit("expected local_heavy_lane_execution_mode=opt_in")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
PY

set +e
invalid_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected live validation script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
zero_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds 0; } 2>&1)"
zero_budget_code=$?
set -e
if [ "$zero_budget_code" -eq 0 ]; then
  echo "expected live validation script to reject zero max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$zero_budget_output" | grep -q 'max-seconds must be greater than zero'; then
  echo "expected deterministic zero max-seconds marker" >&2
  exit 1
fi

echo "message proof anchoring live validation tests passed."
