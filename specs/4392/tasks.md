# Tasks — #4392

Status: Implemented

T1 (RED)
- Add failing assertions for websocket policy `reason_codes_value` on success and failure paths.

T2 (RED)
- Add failing tamper case for missing required websocket taxonomy field and deterministic required-field reason mapping.

T3 (RED)
- Add failing integration assertion for websocket contract-lane policy `reason_codes_value=none` output.

T4 (Regression)
- Re-run websocket policy and contract-lane suites after #4393 implementation.
