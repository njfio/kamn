#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_runtime_commit_live_finality_evidence_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_runtime_commit_live_finality_evidence_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
FOUNDATION_DOC="$ROOT_DIR/docs/foundation/kolme-runtime-commit-client.md"
CI_STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local runtime-commit live finality evidence contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local runtime-commit live evidence policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local runtime-commit live finality evidence contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local runtime-commit live finality evidence contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/local_runtime_commit_live_finality_evidence_contract_lane.py",
]:
    raise SystemExit("expected runtime-commit live finality evidence manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected runtime-commit live finality evidence contract implementation to exist" >&2
  exit 1
fi

required_markers=(
  "run_local_runtime_commit_live_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "submit_evidence_marker_present"
  "finality_evidence_marker_present"
  "Regression: #2099"
)
for marker in "${required_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected runtime-commit live finality evidence contract implementation marker: $marker" >&2
    exit 1
  fi
done

required_doc_markers=(
  "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "submit_evidence_marker_present"
  "finality_evidence_marker_present"
)
for marker in "${required_doc_markers[@]}"; do
  if ! grep -q "$marker" "$DOC_FILE"; then
    echo "expected Kolme devnet ops documentation marker: $marker" >&2
    exit 1
  fi
  if ! grep -q "$marker" "$FOUNDATION_DOC"; then
    echo "expected runtime commit foundation documentation marker: $marker" >&2
    exit 1
  fi
  if ! grep -q "$marker" "$CI_STRATEGY_DOC"; then
    echo "expected CI strategy documentation marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference local runtime-commit finality evidence contract lane" >&2
  exit 1
fi

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if summary.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
    raise SystemExit("unexpected runtime-commit live finality evidence summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected runtime-commit live finality evidence summary status ok")
if summary.get("reason_code") != "live_runtime_commit_and_finality_commands_passed":
    raise SystemExit("expected runtime-commit live finality evidence summary reason code")
if summary.get("finality_enabled") is not True:
    raise SystemExit("expected finality_enabled=true in runtime-commit live finality evidence summary")
if summary.get("submit_evidence_marker_present") is not True:
    raise SystemExit("expected submit_evidence_marker_present=true in runtime-commit live finality evidence summary")
if summary.get("finality_evidence_marker_present") is not True:
    raise SystemExit("expected finality_evidence_marker_present=true in runtime-commit live finality evidence summary")

if policy.get("schema_version") != "kamn.kolme.local-runtime-commit-live-policy-report.v1":
    raise SystemExit("unexpected runtime-commit live finality evidence policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected runtime-commit live finality evidence policy final_decision GO")
PY

echo "local runtime-commit live finality evidence contract lane tests passed."
