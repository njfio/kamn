#!/usr/bin/env python3
"""Reusable process harness primitives for local integration lanes."""

from __future__ import annotations

from dataclasses import dataclass
import json
import socket
import subprocess
import time
from pathlib import Path
from typing import Any, TextIO
from urllib.error import URLError
from urllib.request import urlopen


class ProcessHarnessError(RuntimeError):
    """Raised when process harness invariants are violated."""


@dataclass
class PortReservation:
    """Tracks a reserved localhost TCP port."""

    label: str
    port: int
    host: str
    _socket: socket.socket

    def release(self) -> None:
        """Release the reservation socket."""
        if self._socket.fileno() >= 0:
            self._socket.close()


@dataclass
class ManagedProcess:
    """Tracks a spawned process and its evidence metadata."""

    name: str
    command: list[str]
    log_file: Path
    process: subprocess.Popen[str]
    started_at_epoch: int
    _log_handle: TextIO

    def close_log(self) -> None:
        """Close the attached log file handle."""
        if not self._log_handle.closed:
            self._log_handle.close()


class ProcessHarness:
    """Reusable lifecycle harness for local process orchestration."""

    def __init__(self, *, root_dir: Path) -> None:
        self.root_dir = root_dir
        self._reservations: dict[str, PortReservation] = {}
        self._processes: dict[str, ManagedProcess] = {}

    def __enter__(self) -> ProcessHarness:
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        self.close()

    def reserve_port(
        self,
        label: str,
        *,
        host: str = "127.0.0.1",
        start_port: int = 30000,
        end_port: int = 60000,
    ) -> int:
        """Reserve a deterministic local TCP port for later process startup."""
        if label in self._reservations:
            raise ProcessHarnessError(f"port label already reserved: {label}")
        if start_port > end_port:
            raise ProcessHarnessError("start_port must be <= end_port")

        reserved_ports = {reservation.port for reservation in self._reservations.values()}
        for port in range(start_port, end_port + 1):
            if port in reserved_ports:
                continue
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                sock.bind((host, port))
                sock.listen(1)
            except OSError:
                sock.close()
                continue

            self._reservations[label] = PortReservation(
                label=label,
                port=port,
                host=host,
                _socket=sock,
            )
            return port

        raise ProcessHarnessError(
            f"no available port found for label '{label}' in range {start_port}-{end_port}"
        )

    def release_port(self, label: str) -> None:
        """Release a previously reserved port label."""
        reservation = self._reservations.pop(label, None)
        if reservation is None:
            raise ProcessHarnessError(f"unknown port reservation label: {label}")
        reservation.release()

    def start_process(
        self,
        name: str,
        command: list[str],
        *,
        log_file: Path,
        cwd: Path | None = None,
        env: dict[str, str] | None = None,
        release_port_labels: tuple[str, ...] = (),
    ) -> ManagedProcess:
        """Start a process, capture stdout/stderr to a log, and track lifecycle."""
        if name in self._processes:
            raise ProcessHarnessError(f"process already running for name: {name}")
        for label in release_port_labels:
            self.release_port(label)

        log_file.parent.mkdir(parents=True, exist_ok=True)
        log_handle = log_file.open("w", encoding="utf-8")
        try:
            process = subprocess.Popen(
                command,
                cwd=str(cwd or self.root_dir),
                env=env,
                stdout=log_handle,
                stderr=subprocess.STDOUT,
                text=True,
            )
        except OSError as error:
            log_handle.close()
            raise ProcessHarnessError(
                f"failed to start process '{name}': {error}"
            ) from error

        managed = ManagedProcess(
            name=name,
            command=list(command),
            log_file=log_file,
            process=process,
            started_at_epoch=int(time.time()),
            _log_handle=log_handle,
        )
        self._processes[name] = managed
        return managed

    def wait_for_http_ready(
        self,
        url: str,
        *,
        timeout_seconds: int,
        expected_status: int = 200,
        interval_seconds: float = 0.1,
    ) -> bool:
        """Poll an HTTP endpoint until readiness or timeout."""
        if timeout_seconds <= 0:
            raise ProcessHarnessError("timeout_seconds must be greater than zero")

        deadline = time.monotonic() + float(timeout_seconds)
        while time.monotonic() < deadline:
            try:
                with urlopen(url, timeout=1) as response:
                    if int(response.status) == expected_status:
                        return True
            except URLError:
                pass
            time.sleep(interval_seconds)
        return False

    def stop_process(self, name: str, *, grace_seconds: int = 3) -> dict[str, Any]:
        """Stop a tracked process and return machine-checkable stop evidence."""
        managed = self._processes.pop(name, None)
        if managed is None:
            return {
                "name": name,
                "status": "not_running",
                "reason_code": "process_not_running",
            }

        process = managed.process
        reason_code = "already_exited"
        was_forced = False
        if process.poll() is None:
            process.terminate()
            reason_code = "terminated_gracefully"
            try:
                process.wait(timeout=max(grace_seconds, 1))
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=max(grace_seconds, 1))
                reason_code = "terminated_forced"
                was_forced = True

        managed.close_log()
        return {
            "name": name,
            "status": "stopped",
            "reason_code": reason_code,
            "exit_code": process.poll(),
            "forced": was_forced,
            "pid": process.pid,
            "log_file": str(managed.log_file),
        }

    def stop_all(self, *, grace_seconds: int = 3) -> dict[str, dict[str, Any]]:
        """Stop all tracked processes in reverse-start order."""
        results: dict[str, dict[str, Any]] = {}
        for name in reversed(list(self._processes.keys())):
            results[name] = self.stop_process(name, grace_seconds=grace_seconds)
        return results

    def close(self) -> None:
        """Close all tracked resources."""
        self.stop_all()
        for reservation in list(self._reservations.values()):
            reservation.release()
        self._reservations.clear()


def _validate_evidence_payload(payload: dict[str, Any]) -> None:
    required_fields = (
        "schema_version",
        "status",
        "final_decision",
        "reason_code",
        "ports",
        "processes",
        "artifacts",
    )
    for field_name in required_fields:
        if field_name not in payload:
            raise ProcessHarnessError(f"missing evidence field: {field_name}")

    if not isinstance(payload["schema_version"], str):
        raise ProcessHarnessError("schema_version must be a string")
    if payload["status"] not in ("pass", "fail"):
        raise ProcessHarnessError("status must be pass or fail")
    if payload["final_decision"] not in ("GO", "NO-GO"):
        raise ProcessHarnessError("final_decision must be GO or NO-GO")
    if not isinstance(payload["reason_code"], str):
        raise ProcessHarnessError("reason_code must be a string")

    ports = payload["ports"]
    if not isinstance(ports, dict):
        raise ProcessHarnessError("ports must be an object")
    for label, port in ports.items():
        if not isinstance(label, str):
            raise ProcessHarnessError("ports labels must be strings")
        if not isinstance(port, int) or port <= 0:
            raise ProcessHarnessError("ports values must be positive integers")

    processes = payload["processes"]
    if not isinstance(processes, list):
        raise ProcessHarnessError("processes must be an array")
    for entry in processes:
        if not isinstance(entry, dict):
            raise ProcessHarnessError("process entries must be objects")
        if not isinstance(entry.get("name"), str):
            raise ProcessHarnessError("process entry name must be a string")
        if not isinstance(entry.get("status"), str):
            raise ProcessHarnessError("process entry status must be a string")

    artifacts = payload["artifacts"]
    if not isinstance(artifacts, dict):
        raise ProcessHarnessError("artifacts must be an object")
    for key, value in artifacts.items():
        if not isinstance(key, str) or not isinstance(value, str):
            raise ProcessHarnessError("artifacts entries must map strings to strings")


def write_evidence_report(path: Path, payload: dict[str, Any]) -> None:
    """Write a validated machine-checkable evidence report."""
    _validate_evidence_payload(payload)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def load_evidence_report(path: Path) -> dict[str, Any]:
    """Load and validate an evidence report payload."""
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ProcessHarnessError(f"evidence report is not valid JSON: {error}") from error
    if not isinstance(payload, dict):
        raise ProcessHarnessError("evidence report must be a JSON object")

    _validate_evidence_payload(payload)
    return payload
