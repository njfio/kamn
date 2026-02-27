# Issue 6229 Tasks

- T1 (Red/Baseline): Run current critical-path coverage gate and capture per-target measured coverage.
- T2 (Green/Implementation): Update `.ci/critical-path-coverage-thresholds.json` with staged increases.
- T3 (Green/Docs): Add rationale for each threshold ratchet in architecture/testing docs.
- T4 (Regression): Re-run coverage gate with updated thresholds and verify deterministic pass/fail output.
- T5 (Verification): Map AC/C-cases and close issue with evidence links.
