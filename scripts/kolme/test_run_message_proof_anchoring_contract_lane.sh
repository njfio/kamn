#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_message_proof_anchoring_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_message_proof_anchoring_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/message_proof_anchoring_contract_lane.py"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected message proof anchoring contract runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected message proof anchoring contract dispatcher to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected message proof anchoring contract manifest to exist" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected message proof anchoring contract implementation to exist" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected message proof anchoring contract runner to be a symlink to shared dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_contract_lane_dispatch.sh" ]; then
  echo "expected message proof anchoring contract runner symlink target to be run_contract_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected message proof anchoring contract dispatcher to resolve deterministic manifest path" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected message proof anchoring manifest schema")
if payload.get("lane_id") != "kolme.message_proof_anchoring.contract":
    raise SystemExit("unexpected message proof anchoring manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/message_proof_anchoring_contract_lane.py",
]:
    raise SystemExit("unexpected message proof anchoring manifest contract command")
PY

required_impl_markers=(
  "functional_anchor_submission_advances_broadcast_to_included_with_typed_outcome"
  "integration_anchor_retry_is_duplicate_without_reapplying_state_transition"
  "regression_anchor_conflicting_payload_for_same_message_rejected_fail_closed"
  "performance_anchor_submission_contract_lane_stays_within_budget"
  "kamn.kolme.message-proof-anchoring.contract.v1"
)
for marker in "${required_impl_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected message proof anchoring implementation marker: $marker" >&2
    exit 1
  fi
done

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected message proof anchoring status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected message proof anchoring GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^message_anchor_contract_status=verified$'; then
  echo "expected message anchor contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^lifecycle_alignment_status=verified$'; then
  echo "expected lifecycle alignment marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^conflict_fail_closed_status=verified$'; then
  echo "expected conflict fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^performance_budget_status=verified$'; then
  echo "expected performance budget marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.message-proof-anchoring.contract.v1":
    raise SystemExit("unexpected message proof anchoring contract schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("message_anchor_contract_status") != "verified":
    raise SystemExit("expected message_anchor_contract_status=verified")
if payload.get("lifecycle_alignment_status") != "verified":
    raise SystemExit("expected lifecycle_alignment_status=verified")
if payload.get("conflict_fail_closed_status") != "verified":
    raise SystemExit("expected conflict_fail_closed_status=verified")
if payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")
PY

set +e
invalid_budget_output="$({ bash "$RUNNER" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected runner to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
zero_budget_output="$({ bash "$RUNNER" --max-seconds 0; } 2>&1)"
zero_budget_code=$?
set -e
if [ "$zero_budget_code" -eq 0 ]; then
  echo "expected runner to reject zero max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$zero_budget_output" | grep -q 'max-seconds must be greater than zero'; then
  echo "expected deterministic zero max-seconds marker" >&2
  exit 1
fi

echo "message proof anchoring contract lane tests passed."
