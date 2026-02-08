#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/did/run_did_registry_contract_lane.sh"

if [ ! -x "$SCRIPT" ]; then
  echo "expected did registry contract lane runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$SCRIPT" >"$TMP_OUT"
if ! grep -q "did registry contract lane tests passed." "$TMP_OUT"; then
  echo "expected did registry contract lane success marker" >&2
  exit 1
fi

if ! grep -q "retry_classification_is_deterministic_for_duplicate_submission" "$SCRIPT"; then
  echo "expected did registry lane to include retry classification test coverage" >&2
  exit 1
fi

if ! grep -q "integration_register_retry_and_finality_boundary_is_idempotent" "$SCRIPT"; then
  echo "expected did registry lane to include finality idempotency integration coverage" >&2
  exit 1
fi

if ! grep -q "regression_register_finality_rejects_stale_or_conflicting_updates" "$SCRIPT"; then
  echo "expected did registry lane to include stale/conflict regression coverage" >&2
  exit 1
fi

echo "did registry contract lane script tests passed."
