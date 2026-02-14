#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_did_lifecycle_chain_adapter_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_did_lifecycle_chain_adapter_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/did_lifecycle_chain_adapter_contract_lane.py"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected did lifecycle chain adapter contract runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected did lifecycle chain adapter contract dispatcher to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected did lifecycle chain adapter contract manifest to exist" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected did lifecycle chain adapter contract implementation to exist" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected did lifecycle chain adapter contract runner to be a symlink to shared dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_contract_lane_dispatch.sh" ]; then
  echo "expected did lifecycle chain adapter contract runner symlink target to be run_contract_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected did lifecycle chain adapter contract dispatcher to resolve deterministic manifest path" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected did lifecycle chain adapter manifest schema")
if payload.get("lane_id") != "kolme.did_lifecycle_chain_adapter.contract":
    raise SystemExit("unexpected did lifecycle chain adapter manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/did_lifecycle_chain_adapter_contract_lane.py",
]:
    raise SystemExit("unexpected did lifecycle chain adapter manifest contract command")
PY

required_impl_markers=(
  "functional_lifecycle_chain_submission_through_kolme_adapter_returns_typed_outcome"
  "integration_lifecycle_chain_submission_allows_retry_without_reapplying_mutation"
  "regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload"
  "kamn.kolme.did-lifecycle-chain.contract.v1"
)
for marker in "${required_impl_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected did lifecycle chain adapter implementation marker: $marker" >&2
    exit 1
  fi
done

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected did lifecycle contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected did lifecycle contract GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^lifecycle_chain_contract_status=verified$'; then
  echo "expected lifecycle chain marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^duplicate_retry_status=verified$'; then
  echo "expected duplicate retry marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^conflict_fail_closed_status=verified$'; then
  echo "expected conflict fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.did-lifecycle-chain.contract.v1":
    raise SystemExit("unexpected did lifecycle contract schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("lifecycle_chain_contract_status") != "verified":
    raise SystemExit("expected lifecycle_chain_contract_status=verified")
if payload.get("duplicate_retry_status") != "verified":
    raise SystemExit("expected duplicate_retry_status=verified")
if payload.get("conflict_fail_closed_status") != "verified":
    raise SystemExit("expected conflict_fail_closed_status=verified")
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

echo "did lifecycle chain adapter contract lane tests passed."
