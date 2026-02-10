#!/usr/bin/env python3
"""Unit tests for shared contract-lane helper utilities."""

from __future__ import annotations

import os
import tempfile
import time
import unittest
from pathlib import Path

from contract_lane_helpers import (
    ContractLaneError,
    build_default_bundle_args,
    enforce_runtime_budget,
    require_output_contains,
    run_capture,
    run_go_bundle_policy_pair,
)


class ContractLaneHelpersTests(unittest.TestCase):
    def test_run_capture_returns_exit_code_and_output(self) -> None:
        code, output = run_capture(["python3", "-c", "print('ok')"])
        self.assertEqual(code, 0)
        self.assertIn("ok", output)

    def test_build_default_bundle_args_contains_required_pairs(self) -> None:
        args = build_default_bundle_args(
            output_file="/tmp/example.json",
            pairs=(
                ("--control-id", "CC6.1"),
                ("--ci-fast-gate", "PASS"),
            ),
        )
        self.assertIn("--output-file", args)
        self.assertIn("/tmp/example.json", args)
        self.assertIn("--control-id", args)
        self.assertIn("CC6.1", args)

    def test_require_output_contains_raises_for_missing_marker(self) -> None:
        with self.assertRaises(ContractLaneError):
            require_output_contains("status=ok", expected="final_decision=GO", context="test")

    def test_run_go_bundle_policy_pair_accepts_go_markers(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            generator = temp_path / "generator.sh"
            policy = temp_path / "policy.sh"
            bundle_file = temp_path / "bundle.json"

            generator.write_text(
                "\n".join(
                    (
                        "#!/usr/bin/env bash",
                        "set -euo pipefail",
                        'output_file=""',
                        'while [ \"$#\" -gt 0 ]; do',
                        '  case \"$1\" in',
                        '    --output-file) output_file=\"$2\"; shift 2 ;;',
                        "    *) shift ;;",
                        "  esac",
                        "done",
                        'printf \"{\\\"schema_version\\\":\\\"v1\\\"}\\n\" > \"$output_file\"',
                        "echo \"final_decision=GO\"",
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            policy.write_text(
                "\n".join(
                    (
                        "#!/usr/bin/env bash",
                        "set -euo pipefail",
                        '[ \"$1\" = \"--bundle-file\" ]',
                        '[ -f \"$2\" ]',
                        "echo \"final_decision=GO\"",
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            os.chmod(generator, 0o755)
            os.chmod(policy, 0o755)

            generator_output, policy_output = run_go_bundle_policy_pair(
                root_dir=temp_path,
                generator=generator,
                generator_args=("--output-file", str(bundle_file)),
                policy_checker=policy,
                bundle_file=bundle_file,
            )

            self.assertIn("final_decision=GO", generator_output)
            self.assertIn("final_decision=GO", policy_output)

    def test_run_go_bundle_policy_pair_rejects_missing_go_marker(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            generator = temp_path / "generator.sh"
            policy = temp_path / "policy.sh"
            bundle_file = temp_path / "bundle.json"

            generator.write_text(
                "\n".join(
                    (
                        "#!/usr/bin/env bash",
                        "set -euo pipefail",
                        'printf \"{}\\n\" > \"$2\"',
                        "echo \"status=generated\"",
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            policy.write_text(
                "\n".join(
                    (
                        "#!/usr/bin/env bash",
                        "set -euo pipefail",
                        "echo \"final_decision=GO\"",
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            os.chmod(generator, 0o755)
            os.chmod(policy, 0o755)

            with self.assertRaises(ContractLaneError):
                run_go_bundle_policy_pair(
                    root_dir=temp_path,
                    generator=generator,
                    generator_args=("--output-file", str(bundle_file)),
                    policy_checker=policy,
                    bundle_file=bundle_file,
                )

    def test_enforce_runtime_budget_reports_elapsed_and_detects_overrun(self) -> None:
        elapsed = enforce_runtime_budget(
            lane_name="demo lane",
            started_at=time.monotonic(),
            max_runtime_seconds=60,
        )
        self.assertGreaterEqual(elapsed, 0)

        with self.assertRaises(ContractLaneError):
            enforce_runtime_budget(
                lane_name="demo lane",
                started_at=time.monotonic() - 2.0,
                max_runtime_seconds=0,
            )


if __name__ == "__main__":
    unittest.main()
