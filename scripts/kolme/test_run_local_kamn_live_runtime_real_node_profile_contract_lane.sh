#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_kamn_live_runtime_real_node_profile_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_kamn_live_runtime_real_node_profile_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local KAMN live runtime real-node profile contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local KAMN live runtime real-node profile policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local KAMN live runtime real-node profile contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local KAMN live runtime real-node profile contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_kamn_live_runtime_real_node_profile_contract_lane.py",
]:
    raise SystemExit("expected local KAMN live runtime real-node profile manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local KAMN live runtime real-node profile contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kamn_live_runtime_integration_lane.sh"
  "check_local_kamn_live_runtime_real_node_profile_policy.py"
  "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh"
  "docs/planning/kolme-devnet-ops.md"
  "docs/ci/strategy.md"
  "README.md"
  "Regression: #2139"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local KAMN live runtime real-node profile contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q -- "--runtime-profile real-node" "$docs_file"; then
    echo "expected docs parity to include real-node runtime profile marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_local_kamn_live_runtime_real_node_profile_policy.py" "$docs_file"; then
    echo "expected docs parity to include real-node profile policy checker marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to include real-node profile contract lane marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2139" "$docs_file"; then
    echo "expected docs parity to include real-node profile regression marker in $docs_file" >&2
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
if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
    raise SystemExit("unexpected real-node profile contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected real-node profile contract-lane summary status ok")
if summary.get("reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected dry-run reason code in real-node profile contract-lane summary")
if summary.get("runtime_profile") != "real-node":
    raise SystemExit("expected runtime_profile=real-node in real-node profile contract-lane summary")
contracts = summary.get("contracts", {})
if contracts.get("runtime_profile") != "real-node":
    raise SystemExit("expected contracts.runtime_profile=real-node in real-node profile contract-lane summary")
if policy.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1":
    raise SystemExit("unexpected real-node profile contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected real-node profile contract-lane policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected no policy reason codes for real-node profile contract-lane dry-run composition")
PY

echo "local KAMN live runtime real-node profile contract lane tests passed."
