---
type: Practice
title: Hardcoded Credentials Detection — Recognition Patterns
description: Recognize and block hardcoded credentials by their format patterns — AWS (AKIA*), Stripe (sk_live_*), Google (AIza*), GitHub (ghp_*), JWT (eyJ*), private key blocks, and connection strings.
tags: [devsecops, security, credentials, secrets, detection, aws, stripe, github, jwt]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: codeguard-1-hardcoded-credentials
    resource: ".devin/rules/codeguard-1-hardcoded-credentials.md"
    title: "job-aide"
  - id: codeguard-0-devops-ci-cd-containers
    resource: ".devin/rules/codeguard-0-devops-ci-cd-containers.md"
    title: "job-aide (secrets section)"
---


# Hardcoded Credentials Detection — Recognition Patterns

## Failure Mode

Storing secrets — passwords, API keys, tokens, private keys, connection
strings — directly in source code is a critical security defect. Any
credential that appears in source code must be treated as compromised. The
codebase should be treated as public and untrusted, even when it is private,
because credentials leak via logs, error messages, build artifacts, shared
terminals, and git history.

## Never Hardcode These Value Types

- Database passwords, user passwords, admin passwords
- API keys, secret keys, access tokens, refresh tokens
- Private keys, certificates, signing keys
- Connection strings containing credentials
- OAuth client secrets, webhook secrets
- Any credential that grants access to external services

## Recognition Patterns

Learn to spot these credential formats in code reviews and automated scans:

### AWS Keys

Start with `AKIA`, `AGPA`, `AIDA`, `AROA`, `AIPA`, `ANPA`, `ANVA`, `ASIA`
followed by additional characters.

```
AKIAIOSFODNN7EXAMPLE
```

### Stripe Keys

Start with `sk_live_`, `pk_live_`, `sk_test_`, or `pk_test_`.

```
sk_live_51H8y2y...
```

### Google API Keys

Start with `AIza` followed by 35 characters.

```
AIzaSyA...
```

### GitHub Tokens

Start with `ghp_`, `gho_`, `ghu_`, `ghs_`, or `ghr_`.

```
ghp_1234567890abcdef...
```

### JWT Tokens

Three base64 sections separated by dots, starting with `eyJ`.

```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIx...
```

### Private Key Blocks

Any text between `-----BEGIN` and `-----END PRIVATE KEY-----` (or
`-----BEGIN RSA PRIVATE KEY-----`, `-----BEGIN EC PRIVATE KEY-----`).

```
-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQD...
-----END PRIVATE KEY-----
```

### Connection Strings

URLs with embedded credentials, such as `mongodb://user:pass@host` or
`postgresql://user:pass@host:5432/db`.

```
mongodb://admin:s3cr3t@10.0.0.1:27017/prod
```

## Warning Signs in Code

Beyond explicit patterns, watch for these indicators:

- Variable names containing: `password`, `secret`, `key`, `token`, `auth`
- Long random-looking strings with no clear purpose
- Base64-encoded strings near authentication code
- Any string that grants access to external services

## Practice

Never store credentials in source code. Use secure alternatives:

1. **Runtime secret retrieval**: Fetch credentials at runtime from a vault,
   KMS, or secret manager (HashiCorp Vault, AWS Secrets Manager, Doppler).
2. **Environment variables**: Inject secrets via environment variables at
   runtime, never bake them into images or commit them to `.env` files.
3. **BuildKit secret mounts**: Use `--mount=type=secret` in Dockerfiles so
   credentials never bake into image layers (see
   [buildkit-secrets](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/buildkit-secrets.md) in the
   container-best-practices bundle).
4. **Ansible vault**: Store secrets in encrypted vault files, never in
   plaintext group_vars (see [security-audit-playbook](/security-audit-playbook.md)).

Any credential found in source code must be **rotated immediately** — it is
already compromised. Removing it from the current code does not remove it
from git history.
