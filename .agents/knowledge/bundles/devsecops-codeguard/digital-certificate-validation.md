---
type: Practice
title: Digital Certificate Validation — X.509 Sanity Checks
description: Validate X.509 certificates for expiration (notAfter), key strength (RSA≥2048, EC≥P-256), signature algorithm (SHA-2 only), and self-signed status before trusting them in production.
tags: [devsecops, security, certificates, x509, tls, pki, openssl]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: codeguard-1-digital-certificates
    resource: ".devin/rules/codeguard-1-digital-certificates.md"
    title: "job-aide"
  - id: codeguard-1-crypto-algorithms
    resource: ".devin/rules/codeguard-1-crypto-algorithms.md"
    title: "job-aide (banned signature algorithms)"
---


# Digital Certificate Validation — X.509 Sanity Checks

## Failure Mode

X.509 certificates that are expired, use weak keys, are signed with broken
algorithms, or are unintentionally self-signed cause TLS connection failures
and security vulnerabilities. Expired certificates are rejected by clients;
weak keys are vulnerable to factorization; SHA-1 signatures allow certificate
forgery; self-signed certificates in production break the trust chain.

## Identifying Certificate Data

Scan for certificate data using these heuristics:

- **PEM-encoded strings**: Multi-line string literals beginning with
  `-----BEGIN CERTIFICATE-----` and ending with `-----END CERTIFICATE-----`.
- **File operations**: Reads on files with extensions `.pem`, `.crt`, `.cer`,
  `.der`.
- **Library calls**: `PEM_read_X509` (OpenSSL),
  `cryptography.x509.load_pem_x509_certificate` (Python),
  `CertificateFactory` (Java), `tls.LoadX509KeyPair` (Go).

## Check 1: Expiration Status

| Condition | Severity |
|-----------|----------|
| `notAfter` is in the past | CRITICAL VULNERABILITY |
| `notBefore` is in the future | Warning |

**Report**: `This certificate expired on [YYYY-MM-DD]. It is no longer valid
and will be rejected by clients, causing connection failures. It must be
renewed and replaced immediately.`

## Check 2: Public Key Strength

| Condition | Severity |
|-----------|----------|
| RSA modulus < 2048 bits | High-Priority Warning |
| EC curve < P-256 (e.g. `secp192r1`, `P-192`, `P-224`) | High-Priority Warning |

**Report**: `The certificate's public key is cryptographically weak
([Algorithm], [Key Size]). Keys of this strength are vulnerable to
factorization or discrete logarithm attacks. The certificate should be
re-issued using at least an RSA 2048-bit key or an ECDSA key on a P-256 (or
higher) curve.`

### Minimum Key Strengths

| Algorithm | Minimum |
|-----------|---------|
| RSA | 2048 bits |
| ECDSA | P-256 (secp256r1) |
| Ed25519 | 255 bits (inherently meets requirement) |

## Check 3: Signature Algorithm

| Condition | Severity |
|-----------|----------|
| Signature uses MD5 or SHA-1 (e.g. `md5WithRSAEncryption`, `sha1WithRSAEncryption`) | High-Priority Warning |

**Report**: `The certificate is signed with the insecure algorithm
'[Algorithm]'. This makes it vulnerable to collision attacks, potentially
allowing for certificate forgery. It must be re-issued using a signature based
on the SHA-2 family (e.g. sha256WithRSAEncryption).`

Only SHA-2 family signature algorithms are acceptable:
`sha256WithRSAEncryption`, `sha384WithRSAEncryption`, `sha512WithRSAEncryption`,
`ecdsa-with-SHA256`, `ecdsa-with-SHA384`, `ecdsa-with-SHA512`,
`ed25519`.

## Check 4: Self-Signed Detection

| Condition | Severity |
|-----------|----------|
| `Issuer` and `Subject` fields are identical | Informational |

**Report**: `This is a self-signed certificate. Ensure this is intentional
and only used for development, testing, or internal services where trust is
explicitly configured. Self-signed certificates should never be used for
public-facing production systems as they will not be trusted by browsers or
standard clients.`

Self-signed certificates are acceptable for:
- Development and testing environments
- Internal services with explicitly configured trust (mTLS between services)
- Root CAs (which are self-signed by definition)

They are **not** acceptable for public-facing production TLS endpoints.

## Example: Flagging an Expired Certificate

```python
# Certificate for connecting to legacy_service
LEGACY_CERT = """
-----BEGIN CERTIFICATE-----
MIIC... (data for a certificate that expired on 2024-12-01) ...
-----END CERTIFICATE-----
"""
```

> CRITICAL VULNERABILITY: The certificate stored in `LEGACY_CERT` is invalid.
> - Reason: It expired on 2024-12-01.
> - Impact: Any TLS connection attempting to use this certificate will fail.
> - Action: Renew immediately and update the variable with new PEM data.

## Example: Flagging a Weak Key

```go
// Load the server certificate from disk
cert, err := tls.LoadX509KeyPair("server.crt", "server.key")
// Assume server.crt contains a 1024-bit RSA key.
```

> High-Priority Warning: The certificate loaded from `server.crt` uses a weak
> public key.
> - Reason: RSA key with 1024-bit modulus.
> - Impact: Vulnerable to modern cryptanalytic attacks.
> - Action: Regenerate with at least 2048-bit RSA or a modern elliptic curve.
