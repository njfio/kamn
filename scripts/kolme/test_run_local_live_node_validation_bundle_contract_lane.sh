#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_live_node_validation_bundle_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_live_node_validation_bundle_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_live_node_validation_bundle_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_live_node_validation_bundle_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local live-node validation bundle contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local live-node validation bundle policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local live-node validation bundle contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local live-node validation bundle contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_live_node_validation_bundle_contract_lane.py",
]:
    raise SystemExit("expected local live-node validation bundle manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local live-node validation bundle contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_live_node_validation_bundle_lane.sh"
  "check_local_live_node_validation_bundle_policy.py"
  "run_local_live_node_validation_bundle_contract_lane.sh"
  "rollback_evidence_file"
  "recovery_evidence_file"
  "rollback_evidence_file_missing"
  "contracts.rollback_recovery_artifact_lineage_required=true"
  "contracts.process_lifecycle_rollback_evidence_option=--rollback-evidence-file"
  "contracts.process_lifecycle_recovery_evidence_option=--recovery-evidence-file"
  "docs/planning/kolme-devnet-ops.md"
  "docs/ci/strategy.md"
  "README.md"
  "Regression: #2134"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local live-node validation bundle contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q "run_local_live_node_validation_bundle_lane.sh" "$docs_file"; then
    echo "expected docs parity to include bundle runner marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_local_live_node_validation_bundle_policy.py" "$docs_file"; then
    echo "expected docs parity to include bundle policy checker marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "run_local_live_node_validation_bundle_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to include bundle contract lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "rollback_evidence_file" "$docs_file"; then
    echo "expected docs parity to include rollback evidence marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "recovery_evidence_file" "$docs_file"; then
    echo "expected docs parity to include recovery evidence marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "rollback_evidence_file_missing" "$docs_file"; then
    echo "expected docs parity to include rollback lineage reason marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.rollback_recovery_artifact_lineage_required=true" "$docs_file"; then
    echo "expected docs parity to include rollback/recovery lineage contract marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.process_lifecycle_rollback_evidence_option=--rollback-evidence-file" "$docs_file"; then
    echo "expected docs parity to include process lifecycle rollback option contract marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "contracts.process_lifecycle_recovery_evidence_option=--recovery-evidence-file" "$docs_file"; then
    echo "expected docs parity to include process lifecycle recovery option contract marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2134" "$docs_file"; then
    echo "expected docs parity to include bundle workflow regression marker in $docs_file" >&2
    exit 1
  fi
done

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY_REPORT" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-summary.v1":
    raise SystemExit("unexpected local live-node validation bundle contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected local live-node validation bundle contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry_run_no_commands_executed reason code in bundle contract-lane summary")
if summary.get("ci_fast_gate_eligible") is not False:
    raise SystemExit("expected local-only fast-gate exclusion marker in bundle contract-lane summary")
rollback_evidence_file = summary.get("rollback_evidence_file")
if not isinstance(rollback_evidence_file, str) or not rollback_evidence_file:
    raise SystemExit("expected rollback_evidence_file marker in bundle contract-lane summary")
recovery_evidence_file = summary.get("recovery_evidence_file")
if not isinstance(recovery_evidence_file, str) or not recovery_evidence_file:
    raise SystemExit("expected recovery_evidence_file marker in bundle contract-lane summary")
if rollback_evidence_file not in summary.get("artifact_paths", []):
    raise SystemExit("expected rollback evidence artifact path marker in bundle contract-lane summary")
if recovery_evidence_file not in summary.get("artifact_paths", []):
    raise SystemExit("expected recovery evidence artifact path marker in bundle contract-lane summary")
contracts = summary.get("contracts", {})
if contracts.get("ci_fast_gate_scope") != "local-only":
    raise SystemExit("expected local-only fast-gate scope contract marker in bundle contract-lane summary")
if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected runtime provider contract marker in bundle contract-lane summary")
if contracts.get("rollback_recovery_artifact_lineage_required") is not True:
    raise SystemExit("expected rollback/recovery lineage required contract marker in bundle contract-lane summary")
if contracts.get("process_lifecycle_rollback_evidence_option") != "--rollback-evidence-file":
    raise SystemExit("expected process lifecycle rollback option contract marker in bundle contract-lane summary")
if contracts.get("process_lifecycle_recovery_evidence_option") != "--recovery-evidence-file":
    raise SystemExit("expected process lifecycle recovery option contract marker in bundle contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-policy-report.v1":
    raise SystemExit("unexpected local live-node validation bundle contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected local live-node validation bundle contract-lane policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no policy reason codes for bundle contract-lane dry-run composition")
PY

echo "local live-node validation bundle contract lane tests passed."
