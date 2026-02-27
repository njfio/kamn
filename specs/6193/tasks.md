# Tasks: Issue 6193 - Signer Adapter Must Not Clone Private Key Material

- Issue: #6193
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add/extend source contract test to fail on signer adapter `Clone` derive.
- [x] T2 (GREEN): remove `Clone` derive from `KolmeForkSecp256k1SignerAdapter`.
- [x] T3 (REGRESSION): run signer-focused node test lanes.
- [x] T4 (VERIFY): run boundary contract lane plus scoped fmt/clippy.
