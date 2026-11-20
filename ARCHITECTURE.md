# PaySwitch Architecture & Invariants Specification

## 1. System Abstract

PaySwitch is a deterministic, low-latency financial switching and gross settlement engine designed for sovereign cross-border settlement in emerging economies. The engine addresses structural inefficiencies in traditional correspondent banking:

1. Settlement Delays (3-5 days in SWIFT reduced to sub-millisecond local execution).
2. Correspondent Fee Churn (3-7% reduced to predictable sub-percent basis points).
3. Reconciliation Drift (Float loss eliminated through integer-only double-entry invariants).

---

## 2. Mathematical Double-Entry Invariants

The ledger enforces strict conservation of monetary value across all transactions:

$$\sum \Delta \text{Debits} \equiv \sum \Delta \text{Credits}$$

For every financial transfer $T$:

$$\text{Balance}_{\text{debit}}(t_1) = \text{Balance}_{\text{debit}}(t_0) - A$$
$$\text{Balance}_{\text{credit}}(t_1) = \text{Balance}_{\text{credit}}(t_0) + A$$

Where:
- $A \in \mathbb{N}^+$ is an integer atomic monetary amount ($A > 0$).
- Overdraft constraint: $\text{Balance}_{\text{debit}}(t_0) \ge A$.
- Currency segregation: $\text{Currency}(D) \equiv \text{Currency}(C) \equiv \text{Currency}(T)$.

Floating-point representations (IEEE 754) are strictly disallowed across all computational pathways to avoid non-associative rounding errors.

---

## 3. Cryptographic Merkle Audit Chaining

Transactions are sequentially hashed into a tamper-evident cryptographic log:

$$H_n = \text{SHA256}(H_{n-1} \parallel \text{TransferID} \parallel \text{DebitID} \parallel \text{CreditID} \parallel \text{Amount} \parallel \text{Currency})$$

Any modification of historical records invalidates all subsequent hashes, providing instant mathematical proof of ledger integrity.

---

## 4. Cross-Border FX Settlement Pipeline

The cross-border route engine converts inbound Nostro reserves to domestic merchant receivables:

$$\text{Fee} = \left\lfloor \frac{A_{\text{in}} \cdot \text{bps}}{10000} \right\rfloor$$
$$\text{Net}_{\text{source}} = A_{\text{in}} - \text{Fee}$$
$$A_{\text{target}} = \left\lfloor \frac{\text{Net}_{\text{source}} \cdot R_{\text{num}}}{R_{\text{den}}} \right\rfloor$$

All arithmetic is executed using 128-bit unsigned integer operations with saturating and bounded division.

---

## 5. Storage Durability (Write-Ahead Log)

Before committing any ledger mutation to memory:
1. Operation serialized to binary buffer.
2. 32-bit CRC checksum computed.
3. Framed record appended to sequential disk WAL:
   `[Magic: 4 bytes][Length: 4 bytes][CRC32: 4 bytes][Payload bytes]`
4. Hard disk sync completed before returning success to caller.
