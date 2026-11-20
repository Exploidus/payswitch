# PaySwitch Project Governance

This document describes how architectural evolution, release management, and core decisions are governed in the PaySwitch ecosystem.

---

## 1. Project Stewardship

PaySwitch is maintained and guided by Exploidus Labs, founded and led by Rahad Bhuiya as the project architect.

The core maintainer holds ultimate stewardship over:
- Core engine stability and invariant enforcement.
- Release tagging and cryptographic binary signing.
- Dual commercial licensing and institutional partnership arrangements.

---

## 2. Decision-Making Framework

PaySwitch follows a consensus-driven approach based on technical proofs and benchmark metrics:

1. **Architectural RFCs:** Significant modifications to the double-entry model, WAL framing, or ISO adapters require an architectural proposal.
2. **Benchmark Validation:** No modification that causes a regression in transaction throughput or allocation overhead will be approved.
3. **Security Precedence:** Security, balance conservation, and non-repudiation take strict precedence over operational convenience or feature additions.

---

## 3. Releases and Semantic Versioning

PaySwitch strictly follows Semantic Versioning (`MAJOR.MINOR.PATCH`):
- `MAJOR`: Breaking changes to the ledger storage format, WAL binary layout, or public REST API.
- `MINOR`: New currency adapters, enhanced clearing rails, or backward-compatible API additions.
- `PATCH`: Security fixes, internal algorithmic optimizations, and invariant verification enhancements.
