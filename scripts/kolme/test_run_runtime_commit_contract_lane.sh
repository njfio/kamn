#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST_RUNNER="$ROOT_DIR/scripts/framework/run_manifest_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_runtime_commit_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/runtime_commit_contract_lane.py"
PARITY_CHECKER="$ROOT_DIR/scripts/kolme/check_runtime_commit_decomposition_parity_matrix.py"
PARITY_MATRIX="$ROOT_DIR/fixtures/kolme_compatibility/runtime_commit_decomposition_parity_matrix.json"
RUNTIME_COMMIT_MAX_SECONDS="${KAMN_KOLME_RUNTIME_COMMIT_CONTRACT_TEST_MAX_SECONDS:-360}"

case "$RUNTIME_COMMIT_MAX_SECONDS" in
  ''|*[!0-9]*)
    echo "KAMN_KOLME_RUNTIME_COMMIT_CONTRACT_TEST_MAX_SECONDS must be a positive integer" >&2
    exit 1
    ;;
  0)
    echo "KAMN_KOLME_RUNTIME_COMMIT_CONTRACT_TEST_MAX_SECONDS must be a positive integer" >&2
    exit 1
    ;;
esac

if [ ! -x "$MANIFEST_RUNNER" ]; then
  echo "expected manifest runner to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected Kolme runtime commit contract lane manifest to exist" >&2
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
    "scripts/kolme/contracts/runtime_commit_contract_lane.py",
]:
    raise SystemExit("expected runtime commit manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected Kolme runtime commit contract implementation to exist" >&2
  exit 1
fi
if ! grep -q '"--no-run"' "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to prebuild test executables before timing" >&2
  exit 1
fi
if ! grep -q '"--message-format=json"' "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to resolve prebuilt test executables from Cargo JSON" >&2
  exit 1
fi
if ! grep -q "compiler-artifact" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to parse compiler-artifact executable paths" >&2
  exit 1
fi
if ! grep -q "KAMN_KOLME_RUNTIME_COMMIT_MAX_SECONDS" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to support local budget override" >&2
  exit 1
fi
if ! grep -q "KAMN_KOLME_RUNTIME_COMMIT_TARGET_DIR" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to expose an isolated target dir override" >&2
  exit 1
fi
if ! grep -q "CARGO_TARGET_DIR" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to run Cargo in an isolated target dir" >&2
  exit 1
fi
if ! grep -q "timeout=" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to bound prebuilt test subprocesses" >&2
  exit 1
fi
if ! grep -q "subprocess.TimeoutExpired" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to report timed out prebuilt tests" >&2
  exit 1
fi
if ! grep -q "DEFAULT_MAX_SECONDS = 360" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit manifest lane to default to local pre-push budget" >&2
  exit 1
fi

if [ ! -x "$PARITY_CHECKER" ]; then
  echo "expected runtime commit decomposition parity checker to be executable" >&2
  exit 1
fi

if [ ! -f "$PARITY_MATRIX" ]; then
  echo "expected runtime commit decomposition parity matrix fixture to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "kolme_runtime_commit_client"
  "kolme_runtime_commit_finality"
  "run_runtime_commit_contract_lane.sh"
  "check_runtime_commit_decomposition_parity_matrix.py"
  "runtime_commit_decomposition_parity_matrix.json"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected Kolme runtime commit contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

if ! grep -q "fixtures/kolme_commit/runtime_commit_request_cases.txt" "$CONTRACT_IMPL"; then
  echo "expected Kolme runtime commit contract implementation to include fixture coverage" >&2
  exit 1
fi

set +e
lane_output="$(
  KAMN_KOLME_RUNTIME_COMMIT_MAX_SECONDS="$RUNTIME_COMMIT_MAX_SECONDS" \
    bash "$MANIFEST_RUNNER" --manifest "$MANIFEST" --phase contract 2>&1
)"
lane_code=$?
set -e
if [ "$lane_code" -ne 0 ]; then
  printf '%s\n' "$lane_output" >&2
  exit "$lane_code"
fi
if ! printf '%s\n' "$lane_output" | grep -q "Kolme runtime commit contract lane tests passed."; then
  echo "expected Kolme runtime commit contract lane success marker" >&2
  exit 1
fi

echo "Kolme runtime commit contract lane script tests passed."
