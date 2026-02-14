#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_onchain_lifecycle_evidence_contract_lane.sh"
BUNDLE_RUNNER="$ROOT_DIR/scripts/kolme/run_onchain_lifecycle_evidence_bundle_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_onchain_lifecycle_evidence_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_onchain_lifecycle_evidence_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/onchain_lifecycle_evidence_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/foundation/kolme-runtime-commit-client.md"
OPS_DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_SUMMARY="$(mktemp)"
TMP_POLICY="$(mktemp)"
trap 'rm -f "$TMP_SUMMARY" "$TMP_POLICY"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected on-chain lifecycle contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$BUNDLE_RUNNER" ]; then
  echo "expected on-chain lifecycle bundle lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected on-chain lifecycle policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected on-chain lifecycle contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected on-chain lifecycle contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/onchain_lifecycle_evidence_contract_lane.py",
]:
    raise SystemExit("expected on-chain lifecycle manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected on-chain lifecycle contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_onchain_lifecycle_evidence_bundle_lane.sh"
  "check_onchain_lifecycle_evidence_policy.py"
  "run_onchain_lifecycle_evidence_contract_lane.sh"
  "validate_continuous_runtime_commit_live.sh"
  "validate_did_lifecycle_chain_adapter_live.sh"
  "validate_message_proof_anchoring_live.sh"
  "aggregate_bundle_lineage_mismatch"
  "finality_lineage_missing"
  "recovery_lineage_missing"
  "docs/foundation/kolme-runtime-commit-client.md"
  "docs/planning/kolme-devnet-ops.md"
  "docs/ci/strategy.md"
  "README.md"
  "Regression: #3249"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected on-chain lifecycle contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$OPS_DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q "run_onchain_lifecycle_evidence_bundle_lane.sh" "$docs_file"; then
    echo "expected docs parity to include on-chain lifecycle bundle lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_onchain_lifecycle_evidence_policy.py" "$docs_file"; then
    echo "expected docs parity to include on-chain lifecycle policy checker marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "run_onchain_lifecycle_evidence_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to include on-chain lifecycle contract lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "aggregate_bundle_lineage_mismatch" "$docs_file"; then
    echo "expected docs parity to include aggregate lineage tamper marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "finality_lineage_missing" "$docs_file"; then
    echo "expected docs parity to include finality lineage marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "recovery_lineage_missing" "$docs_file"; then
    echo "expected docs parity to include recovery lineage marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #3249" "$docs_file"; then
    echo "expected docs parity to include aggregate lifecycle regression marker in $docs_file" >&2
    exit 1
  fi
done

bash "$RUNNER" --output-json "$TMP_SUMMARY" --policy-output-json "$TMP_POLICY" >/dev/null

python3 - "$TMP_SUMMARY" "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if summary.get("schema_version") != "kamn.kolme.onchain-lifecycle-evidence-bundle.v1":
    raise SystemExit("unexpected on-chain lifecycle summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected on-chain lifecycle summary status ok")
if summary.get("final_decision") != "GO":
    raise SystemExit("expected on-chain lifecycle summary final_decision GO")
if summary.get("finality_lineage_status") != "verified":
    raise SystemExit("expected on-chain lifecycle finality_lineage_status=verified")
if summary.get("recovery_lineage_status") != "verified":
    raise SystemExit("expected on-chain lifecycle recovery_lineage_status=verified")

linked_artifacts = summary.get("linked_artifacts")
if not isinstance(linked_artifacts, list) or len(linked_artifacts) != 3:
    raise SystemExit("expected three linked artifacts in on-chain lifecycle summary")
artifact_ids = {item.get("id") for item in linked_artifacts if isinstance(item, dict)}
if artifact_ids != {"did_lifecycle", "message_proof", "runtime_commit"}:
    raise SystemExit("expected did/message/runtime linked artifacts in on-chain lifecycle summary")
for item in linked_artifacts:
    if not isinstance(item, dict):
        raise SystemExit("expected linked artifact entries to be objects")
    if item.get("sha256", "") == "":
        raise SystemExit("expected linked artifact sha256 marker")
    if item.get("status") != "pass":
        raise SystemExit("expected linked artifact status=pass")
    if item.get("final_decision") != "GO":
        raise SystemExit("expected linked artifact final_decision=GO")
    if item.get("finality_marker_status") != "verified":
        raise SystemExit("expected linked artifact finality marker status verified")
    if item.get("recovery_marker_status") != "verified":
        raise SystemExit("expected linked artifact recovery marker status verified")

if policy.get("schema_version") != "kamn.kolme.onchain-lifecycle-evidence-policy-report.v1":
    raise SystemExit("unexpected on-chain lifecycle policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected on-chain lifecycle policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no policy reason codes for on-chain lifecycle dry-run composition")
PY

echo "on-chain lifecycle evidence contract lane tests passed."
