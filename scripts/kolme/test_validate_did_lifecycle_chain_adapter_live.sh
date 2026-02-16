#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/kolme/validate_did_lifecycle_chain_adapter_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected did lifecycle live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected did lifecycle live pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected did lifecycle live GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^did_lifecycle_contract_status=verified$'; then
  echo "expected did lifecycle contract marker" >&2
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
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_reason_code=did_registry_submission_key_conflict$'; then
  echo "expected fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^did_registration_reason_taxonomy_version=kamn.kolme.did-registration-reason-taxonomy.v1$'; then
  echo "expected deterministic did registration reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^did_registration_reason_codes_csv=did_registry_document_did_mismatch,did_registry_submission_key_conflict$'; then
  echo "expected deterministic did registration reason codes marker" >&2
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
if payload.get("schema_version") != "kamn.kolme.did-lifecycle-chain.live-validation.v1":
    raise SystemExit("unexpected did lifecycle live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("did_lifecycle_contract_status") != "verified":
    raise SystemExit("expected did_lifecycle_contract_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("fail_closed_reason_code") != "did_registry_submission_key_conflict":
    raise SystemExit("expected fail_closed_reason_code=did_registry_submission_key_conflict")
if payload.get("did_registration_reason_taxonomy_version") != "kamn.kolme.did-registration-reason-taxonomy.v1":
    raise SystemExit("expected deterministic did_registration_reason_taxonomy_version")
if payload.get("did_registration_reason_codes_csv") != "did_registry_document_did_mismatch,did_registry_submission_key_conflict":
    raise SystemExit("expected deterministic did_registration_reason_codes_csv")
if payload.get("did_registration_reason_codes_value") != "did_registry_submission_key_conflict":
    raise SystemExit("expected did_registration_reason_codes_value=did_registry_submission_key_conflict")
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

echo "did lifecycle chain adapter live validation tests passed."
