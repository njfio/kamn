#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test signer_backend
bash scripts/ci/run_cargo_test_with_quarantine.sh -- cargo test -p kamn-core --test signer_backend_docs

if ! grep -Fq "## Signer Emulator Contract Lanes" docs/foundation/signer-backend-abstraction.md; then
  echo "expected signer emulator contract lane section in signer-backend-abstraction.md" >&2
  exit 1
fi

if ! grep -Fq "Regression: #619" docs/foundation/signer-backend-abstraction.md; then
  echo "expected regression marker for signer emulator contract lane in signer-backend-abstraction.md" >&2
  exit 1
fi

echo "signer emulator contract lane tests passed."
