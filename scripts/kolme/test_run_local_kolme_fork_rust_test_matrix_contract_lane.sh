#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_rust_test_matrix_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_fork_rust_test_matrix_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_fork_rust_test_matrix_contract_lane.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE="$ROOT_DIR/docs/ci/strategy.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_REPO="$(mktemp -d)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY_REPORT"; rm -rf "$TMP_REPO"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local fork rust test matrix contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local fork rust test matrix policy checker to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$RUNNER"; then
  echo "expected local fork rust test matrix contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local fork rust test matrix contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/local_fork_rust_test_matrix_contract_lane.py",
]:
    raise SystemExit("expected local fork rust test matrix manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local fork rust test matrix contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_local_kolme_fork_rust_test_matrix_lane.sh"
  "check_local_kolme_fork_rust_test_matrix_policy.py"
  "evidence_bundle_schema_version=kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1"
  "evidence_bundle"
  "Regression: #1541"
  "Regression: #2329"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected local fork rust test matrix contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

for docs_file in "$DOC_FILE" "$CI_DOC_FILE" "$README_FILE"; do
  if ! grep -q "run_local_kolme_fork_rust_test_matrix_contract_lane.sh" "$docs_file"; then
    echo "expected docs parity to reference fork rust test matrix contract lane in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "check_local_kolme_fork_rust_test_matrix_policy.py" "$docs_file"; then
    echo "expected docs parity to reference fork rust test matrix policy checker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "evidence_bundle_schema_version=kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1" "$docs_file"; then
    echo "expected docs parity to include evidence bundle schema marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "evidence_bundle" "$docs_file"; then
    echo "expected docs parity to include evidence_bundle marker in $docs_file" >&2
    exit 1
  fi
  if ! grep -q "Regression: #2329" "$docs_file"; then
    echo "expected docs parity to include local rust matrix evidence regression marker in $docs_file" >&2
    exit 1
  fi
done

if ! grep -q "Regression: #1541" "$DOC_FILE"; then
  echo "expected Kolme devnet ops doc to include fork rust test matrix regression marker" >&2
  exit 1
fi

git -C "$TMP_REPO" init -q
git -C "$TMP_REPO" checkout -q -b main
git -C "$TMP_REPO" config user.email "ci@example.com"
git -C "$TMP_REPO" config user.name "CI Runner"
cat >"$TMP_REPO/README.md" <<'EOF'
local fork rust matrix contract lane fixture
EOF
git -C "$TMP_REPO" add README.md
git -C "$TMP_REPO" commit -q -m "init matrix contract fixture"
git -C "$TMP_REPO" remote add origin "https://github.com/njfio/kolme_fork.git"

bash "$RUNNER" \
  --checkout-path "$TMP_REPO" \
  --output-json "$TMP_REPORT" \
  --policy-output-json "$TMP_POLICY_REPORT" \
  >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY_REPORT" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if summary.get("schema_version") != "kamn.kolme.local-fork-rust-test-matrix-summary.v1":
    raise SystemExit("unexpected contract-lane summary schema")
if summary.get("status") != "ok":
    raise SystemExit("expected contract-lane summary status ok")
if policy.get("schema_version") != "kamn.kolme.local-fork-rust-test-matrix-policy-report.v1":
    raise SystemExit("unexpected contract-lane policy schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected contract-lane policy final_decision GO")
PY

echo "local fork rust test matrix contract lane tests passed."
