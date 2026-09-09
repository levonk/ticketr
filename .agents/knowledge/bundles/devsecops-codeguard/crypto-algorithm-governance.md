---
type: Practice
title: Crypto Algorithm Governance — Banned Algorithms and Deprecated APIs
description: Ban broken algorithms (MD5, SHA-1, RC4, DES, 3DES, Blowfish), replace deprecated OpenSSL APIs (AES_encrypt, RSA_new, SHA1_Init) with EVP high-level APIs, and use SHA-256, AES-256-GCM, ChaCha20, ECDHE.
tags: [devsecops, security, cryptography, openssl, aes, rsa, sha, evp, tls]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: codeguard-1-crypto-algorithms
    resource: ".devin/rules/codeguard-1-crypto-algorithms.md"
    title: "job-aide"
  - id: codeguard-0-devops-ci-cd-containers
    resource: ".devin/rules/codeguard-0-devops-ci-cd-containers.md"
    title: "job-aide (toolchain hardening)"
---


# Crypto Algorithm Governance — Banned Algorithms and Deprecated APIs

## Failure Mode

Using cryptographically broken or deprecated algorithms compromises
confidentiality, integrity, and authenticity. MD5 and SHA-0 are vulnerable to
collision attacks. RC4, DES, and 3DES have weak key sizes or known
vulnerabilities. Static RSA key exchange provides no forward secrecy. Beyond
algorithms, OpenSSL's low-level APIs (`AES_encrypt`, `RSA_new`, `SHA1_Init`)
are deprecated because they bypass safe defaults and are being removed from
newer OpenSSL versions.

## Banned Algorithms (Strictly Forbidden)

These algorithms are known to be broken and must never be used:

### Hash Algorithms

| Banned | Reason | Use Instead |
|--------|--------|-------------|
| MD2, MD4, MD5 | Collision attacks | SHA-256, SHA-384, SHA-512 |
| SHA-0 | Cryptographically broken | SHA-256, SHA-384, SHA-512 |

### Symmetric Ciphers

| Banned | Reason | Use Instead |
|--------|--------|-------------|
| RC2, RC4 | Weak, known vulnerabilities | AES-128, AES-256, ChaCha20 |
| DES, 3DES | Weak key sizes, deprecated | AES-128, AES-256 |
| Blowfish | Weak key schedule, max 448-bit | AES-256, ChaCha20 |

### Key Exchange

| Banned | Reason | Use Instead |
|--------|--------|-------------|
| Static RSA | No forward secrecy | ECDHE, DHE with proper validation |
| Anonymous Diffie-Hellman | Vulnerable to MITM | ECDHE with authenticated certs |

### Classical

| Banned | Reason |
|--------|--------|
| Vigenère | Not cryptographically secure |

## Deprecated (Legacy/Weak) Algorithms

These are not outright broken but have known weaknesses and must not be used:

| Algorithm | Weakness |
|-----------|----------|
| SHA-1 | Collision attacks (deprecated by all major CAs) |
| AES-CBC | Padding oracle attacks without authenticated encryption |
| AES-ECB | Identical plaintext blocks produce identical ciphertext |
| RSA with PKCS#1 v1.5 padding | Bleichenbacher attacks |
| DHE with weak/common primes | Small subgroup attacks |

## Deprecated OpenSSL APIs — Forbidden

Never use these deprecated low-level functions. Use the EVP high-level APIs
instead, which enforce safe defaults and are maintained across OpenSSL
versions.

### Symmetric Encryption (AES)

| Deprecated | Replacement |
|------------|-------------|
| `AES_encrypt()`, `AES_decrypt()` | `EVP_EncryptInit_ex()`, `EVP_EncryptUpdate()`, `EVP_EncryptFinal_ex()` (and decrypt equivalents) |

### RSA Operations

| Deprecated | Replacement |
|------------|-------------|
| `RSA_new()`, `RSA_up_ref()`, `RSA_free()` | `EVP_PKEY_new()`, `EVP_PKEY_up_ref()`, `EVP_PKEY_free()` |
| `RSA_set0_crt_params()`, `RSA_get0_n()` | EVP key management APIs |

### Hash Functions

| Deprecated | Replacement |
|------------|-------------|
| `SHA1_Init()`, `SHA1_Update()`, `SHA1_Final()` | `EVP_DigestInit_ex()`, `EVP_DigestUpdate()`, `EVP_DigestFinal_ex()` or `EVP_Q_digest()` |

### MAC Operations

| Deprecated | Replacement |
|------------|-------------|
| `CMAC_Init()`, `HMAC()` (especially with SHA1) | `EVP_Q_MAC()` |

### Key Wrapping

| Deprecated | Replacement |
|------------|-------------|
| `AES_wrap_key()`, `AES_unwrap_key()` | EVP key wrapping APIs or implement via EVP encryption |

### Other

| Deprecated | Replacement |
|------------|-------------|
| `DSA_sign()`, `DH_check()` | Corresponding EVP APIs |

## Secure Implementation Pattern

```c
// Secure AES-256-GCM encryption via EVP
EVP_CIPHER_CTX *ctx = EVP_CIPHER_CTX_new();
if (!ctx) handle_error();

if (EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, key, iv) != 1)
    handle_error();

int len, ciphertext_len;
if (EVP_EncryptUpdate(ctx, ciphertext, &len, plaintext, plaintext_len) != 1)
    handle_error();
ciphertext_len = len;

if (EVP_EncryptFinal_ex(ctx, ciphertext + len, &len) != 1)
    handle_error();
ciphertext_len += len;

EVP_CIPHER_CTX_free(ctx);
```

## HMAC with SHA-256 (not SHA-1)

HMAC with SHA-1 is deprecated. Use HMAC with SHA-256 or stronger:

```c
// Instead of HMAC() with SHA1
EVP_Q_MAC(NULL, "HMAC", NULL, "SHA256", NULL,
          key, key_len, data, data_len,
          out, out_size, &out_len);
```

## Code Review Checklist

- [ ] No deprecated SSL/crypto APIs used (`AES_encrypt`, `RSA_new`, `SHA1_Init`, `HMAC` with SHA1)
- [ ] No banned algorithms (MD5, SHA-0, RC4, DES, 3DES, Blowfish)
- [ ] No deprecated algorithms (SHA-1, AES-CBC, AES-ECB, RSA PKCS#1 v1.5)
- [ ] HMAC uses SHA-256 or stronger
- [ ] All crypto operations use EVP high-level APIs
- [ ] Proper error handling for all crypto operations
- [ ] Key material properly zeroed after use
