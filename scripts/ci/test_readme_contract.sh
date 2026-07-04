#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
readme_contract_target_dir="${KAMN_CI_TOOLS_README_CONTRACT_TARGET_DIR:-$ROOT_DIR/target/ci-tools-readme-contract}"
CARGO_TARGET_DIR="$readme_contract_target_dir" cargo test -p kamn-core --test readme_contract_lane -- --nocapture

echo "README contract tests passed."
