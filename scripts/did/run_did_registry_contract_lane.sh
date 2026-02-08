#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions retry_classification_is_deterministic_for_duplicate_submission
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions integration_register_retry_and_finality_boundary_is_idempotent
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions regression_register_finality_rejects_stale_or_conflicting_updates
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test did_registry_transactions_docs

if ! grep -Fq "register_with_retry_guard" docs/foundation/did-registry-transactions.md; then
  echo "expected retry guard contract in did-registry-transactions.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #678" docs/foundation/did-registry-transactions.md; then
  echo "expected regression marker for DID finality guards in did-registry-transactions.md" >&2
  exit 1
fi

echo "did registry contract lane tests passed."
