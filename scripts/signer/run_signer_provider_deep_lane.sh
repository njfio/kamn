#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test signer_backend performance_signer_emulator_bulk_signing_deep_lane -- --ignored
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test signer_backend --test transaction_guards --test transaction_guards_docs

echo "signer provider deep lane tests passed."
