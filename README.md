# PaySwitch

Deterministic, sub-millisecond cross-border financial switching and double-entry settlement engine written in Rust.

PaySwitch provides sovereign financial infrastructure to connect international currencies (USD, EUR, GBP, SGD, USDC) directly to domestic clearing networks (NPSB, BEFTN, RTGS, and mobile financial services such as bKash and Nagad) with zero float loss and cryptographic auditability.

---

## Key Architectural Principles

- **Zero Float Loss:** All calculations use 128-bit unsigned atomic integer units. Floating-point types are strictly prohibited in the financial path.
- **Pure Double-Entry Invariants:** Every transaction enforces debit-credit balance conservation. Overdrafts are rejected at the memory boundary.
- **Cryptographic Audit Chaining:** Continuous SHA-256 Merkle chain guarantees tamper detection across the entire transfer history.
- **Durable Write-Ahead Logging (WAL):** Sequential binary append-only log with CRC-32 checksums ensures complete crash resilience and instant state replay.
- **Compliance & Watchlist Filtering:** Real-time OFAC/UN sanctions screening and velocity limit monitoring.
- **Institutional Standards:** Native adapters for ISO 20022 (`pacs.008`, `pacs.002`) and Bangladesh Bank clearing specifications (NPSB / BEFTN / Binimoy).

---

## Quickstart

### Prerequisites

- Rust 1.80+ (MSRV)
- Cargo

### Building and Testing

```bash
# Run the full test suite (Ledger invariants, WAL persistence, Compliance, API)
cargo test

# Execute the Criterion benchmark suite
cargo bench

# Run the local node
cargo run --bin payswitch-node
```

The node will initialize the Write-Ahead Log, verify state, and expose the HTTP REST API on port `8080`.

---

## API Reference

### Health Check

```http
GET /health
```

### Create Account

```http
POST /api/v1/accounts
Content-Type: application/json

{
  "account_type": "Asset",
  "currency": "USD"
}
```

### Credit Inbound Settlement

```http
POST /api/v1/accounts/{id}/credit
Content-Type: application/json

{
  "amount": 10000000,
  "currency": "USD"
}
```

### Dispatch Cross-Border Transfer

```http
POST /api/v1/settlements/cross-border
Content-Type: application/json

{
  "inbound_nostro_id": "...",
  "platform_fee_id": "...",
  "merchant_account_id": "...",
  "inbound_amount": 250000,
  "source_currency": "USD",
  "target_currency": "BDT",
  "rate_numerator": 12150,
  "rate_denominator": 100,
  "fee_bps": 80,
  "counterparty_name": "Apex Software Exports"
}
```

### Query Audit Tip

```http
GET /api/v1/audit/tip
```

---

## Docker Deployment

```bash
docker build -t payswitch-node .
docker run -p 8080:8080 -v payswitch_data:/home/payswitch/data payswitch-node
```

Or using Docker Compose:

```bash
docker-compose up -d
```

---

## License

PaySwitch is distributed under the GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later) with Dual Commercial Licensing terms. 

For commercial licensing, proprietary embedding, and institutional deployments, contact Exploidus Labs: `rahadbhuiya2021@gmail.com`.
