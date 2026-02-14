#!/usr/bin/env python3
"""Unit and integration tests for reusable process harness primitives."""

from __future__ import annotations

import pathlib
import tempfile
import time
import unittest
import sys

ROOT_DIR = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT_DIR / "scripts"))

from framework.process_harness import (  # noqa: E402
    ProcessHarness,
    ProcessHarnessError,
    load_evidence_report,
    write_evidence_report,
)


class ProcessHarnessTests(unittest.TestCase):
    def test_unit_reserve_port_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            harness = ProcessHarness(root_dir=pathlib.Path(temp_dir))
            first = harness.reserve_port("first", start_port=45000, end_port=45010)
            second = harness.reserve_port("second", start_port=45000, end_port=45010)
            self.assertEqual(first, 45000)
            self.assertEqual(second, 45001)
            harness.close()

    def test_unit_evidence_report_round_trip(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = pathlib.Path(temp_dir) / "evidence.json"
            payload = {
                "schema_version": "kamn.runtime.process-harness-evidence.v1",
                "status": "pass",
                "final_decision": "GO",
                "reason_code": "process_harness_verified",
                "ports": {"api": 19081},
                "processes": [{"name": "api", "status": "stopped"}],
                "artifacts": {"api_log": "/tmp/api.log"},
            }
            write_evidence_report(output_path, payload)
            parsed = load_evidence_report(output_path)
            self.assertEqual(parsed, payload)

    def test_functional_single_process_lifecycle(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            with ProcessHarness(root_dir=ROOT_DIR) as harness:
                port = harness.reserve_port("api", start_port=45100, end_port=45120)
                process = harness.start_process(
                    "api",
                    ["python3", "-m", "http.server", str(port), "--bind", "127.0.0.1"],
                    log_file=temp_path / "api.log",
                    release_port_labels=("api",),
                )
                ready = harness.wait_for_http_ready(f"http://127.0.0.1:{port}/", timeout_seconds=5)
                self.assertTrue(ready)
                stop_result = harness.stop_process("api", grace_seconds=2)
                self.assertEqual(stop_result["status"], "stopped")
                self.assertIsNotNone(process.process.poll())

    def test_integration_multi_process_orchestration(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            with ProcessHarness(root_dir=ROOT_DIR) as harness:
                api_port = harness.reserve_port("api", start_port=45200, end_port=45220)
                ws_port = harness.reserve_port("ws", start_port=45200, end_port=45220)

                harness.start_process(
                    "api",
                    ["python3", "-m", "http.server", str(api_port), "--bind", "127.0.0.1"],
                    log_file=temp_path / "api.log",
                    release_port_labels=("api",),
                )
                harness.start_process(
                    "ws",
                    ["python3", "-m", "http.server", str(ws_port), "--bind", "127.0.0.1"],
                    log_file=temp_path / "ws.log",
                    release_port_labels=("ws",),
                )

                self.assertTrue(
                    harness.wait_for_http_ready(f"http://127.0.0.1:{api_port}/", timeout_seconds=5)
                )
                self.assertTrue(
                    harness.wait_for_http_ready(f"http://127.0.0.1:{ws_port}/", timeout_seconds=5)
                )
                stop_results = harness.stop_all(grace_seconds=2)
                self.assertEqual(stop_results["api"]["status"], "stopped")
                self.assertEqual(stop_results["ws"]["status"], "stopped")

    def test_regression_context_manager_tears_down_on_exception(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            process_ref = None
            try:
                with ProcessHarness(root_dir=ROOT_DIR) as harness:
                    port = harness.reserve_port("api", start_port=45300, end_port=45320)
                    process_ref = harness.start_process(
                        "api",
                        ["python3", "-m", "http.server", str(port), "--bind", "127.0.0.1"],
                        log_file=temp_path / "api.log",
                        release_port_labels=("api",),
                    )
                    self.assertTrue(
                        harness.wait_for_http_ready(
                            f"http://127.0.0.1:{port}/", timeout_seconds=5
                        )
                    )
                    raise RuntimeError("force teardown")
            except RuntimeError:
                pass
            self.assertIsNotNone(process_ref)
            assert process_ref is not None
            self.assertIsNotNone(process_ref.process.poll())

    def test_performance_setup_teardown_within_budget(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = pathlib.Path(temp_dir)
            start = time.monotonic()
            with ProcessHarness(root_dir=ROOT_DIR) as harness:
                port = harness.reserve_port("api", start_port=45400, end_port=45420)
                harness.start_process(
                    "api",
                    ["python3", "-m", "http.server", str(port), "--bind", "127.0.0.1"],
                    log_file=temp_path / "api.log",
                    release_port_labels=("api",),
                )
                ready = harness.wait_for_http_ready(f"http://127.0.0.1:{port}/", timeout_seconds=5)
                self.assertTrue(ready)
            elapsed = time.monotonic() - start
            self.assertLess(
                elapsed,
                8.0,
                msg=f"process harness setup/teardown exceeded budget: {elapsed:.2f}s",
            )

    def test_unit_reject_invalid_evidence_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = pathlib.Path(temp_dir) / "bad-evidence.json"
            with self.assertRaises(ProcessHarnessError):
                write_evidence_report(
                    output_path,
                    {
                        "schema_version": "kamn.runtime.process-harness-evidence.v1",
                        "status": "pass",
                        "final_decision": "GO",
                    },
                )


if __name__ == "__main__":
    unittest.main()
