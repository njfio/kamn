#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_message_proof_anchoring_contract_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected message proof anchoring contract runner to be executable" >&2
  exit 1
fi

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
