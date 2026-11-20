# Security Policy for PaySwitch

Financial infrastructure security and cryptographic integrity are paramount to the PaySwitch network.

---

## Supported Versions

Only the latest release branch receives security updates and vulnerability patches.

| Version | Supported |
| :--- | :--- |
| 0.1.x | Yes |
| < 0.1.0 | No |

---

## Reporting a Vulnerability

If you identify a security issue, vulnerability, or potential exploit in PaySwitch:

1. **Do not disclose the issue publicly.** Do not create public GitHub issues or forum posts describing security weaknesses.
2. Email full technical details, reproduction steps, and impact assessment to:
   `rahadbhuiya2021@gmail.com`
3. We acknowledge vulnerability submissions within 48 hours and work with security researchers on responsible disclosure.

---

## Critical Audit Focus Areas

We especially welcome security review on:
- Double-entry balance calculation bypasses or integer overflow edge cases.
- Write-Ahead Log (WAL) frame tampering or checksum collision vulnerabilities.
- Cryptographic Merkle chain validation bypasses.
- Denial-of-service vectors against the Axum REST ingress layer.
- ISO 20022 and regional switching adapter deserialization exploits.
