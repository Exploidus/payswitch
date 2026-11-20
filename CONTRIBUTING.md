# Contributing to PaySwitch

Thank you for your interest in contributing to PaySwitch sovereign financial infrastructure. 

Because PaySwitch processes cryptographic double-entry ledger transfers and cross-border currency settlements, all contributions must satisfy rigorous engineering standards.

---

## 1. Core Engineering Invariants

Every pull request must preserve the following architectural constraints:

1. **Zero Floating-Point Types:** IEEE 754 floating-point values (`f32`, `f64`) are strictly disallowed in monetary calculation paths. Use `Amount(u128)` atomic units only.
2. **Double-Entry Balance Conservation:** All transfers must balance debit and credit legs identically ($\sum \Delta \text{Debits} \equiv \sum \Delta \text{Credits}$).
3. **Cryptographic Chaining:** Ledger operations must maintain SHA-256 Merkle chain continuity.
4. **Deterministic Storage:** Operations modifying state must be framed and committed to the Write-Ahead Log (WAL) with CRC-32 verification prior to memory updates.
5. **No External Network Calls in Hot Paths:** Ledger mutations must execute synchronously in memory without blocking on third-party network I/O.

---

## 2. Development Workflow

### Setup and Verification

```bash
# Verify code compilation
cargo check

# Run the complete test suite
cargo test

# Run Criterion performance benchmarks
cargo bench
```

All unit tests in `tests/` must pass with zero warnings before any merge.

---

## 3. Pull Request Guidelines

- Provide a concise description of the architectural or algorithmic improvement.
- Include corresponding invariant tests in `tests/`.
- Ensure memory allocations are kept to a minimum in transaction execution paths.
- Avoid introducing unnecessary heavy external dependencies.

---

## 4. Dual Licensing Agreement

By submitting a contribution to PaySwitch, you agree that your contribution will be licensed under the GNU AGPL-3.0-or-later for open-source users and under commercial enterprise terms for proprietary licensees.
