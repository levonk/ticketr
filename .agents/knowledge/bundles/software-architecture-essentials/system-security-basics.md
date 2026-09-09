---
type: Practice
title: System Security Basics
description: Foundational security practices — encrypt, sanitize, least privilege — plus zero-trust, mTLS, OAuth2/OIDC, secrets management, and observability for modern systems.
tags: [architecture, security, encryption, zero-trust, mtls, oauth2, oidc, secrets-management, observability]
date:
  created: "2026-07-24"
  knowledge-basis: "2026-07-24"
  last-used: "2026-07-24"
---

# System Security Basics

## Foundations

- **Encrypt in transit and at rest.** TLS for network; AES-256 for storage.
- **Sanitize inputs** to prevent XSS and SQL injection.
- **Use parameterized queries** — never build SQL with string concatenation.
- **Least privilege** — every service and user gets minimum access.

## Zero-Trust

Never trust a request based on network location. Authenticate and authorize
every request, inside the cluster and outside.

## mTLS

Mutual TLS: both client and server present certificates. The default for
service-to-service security. Usually provided by a service mesh (Istio,
Linkerd).

## OAuth2 and OIDC

- **OAuth2** — delegated authorization (access tokens, refresh tokens).
- **OIDC** — identity layer on top of OAuth2 (ID tokens, userinfo, discovery).

Use OAuth2/OIDC for user-facing auth. For service-to-service, prefer mTLS or
SPIFFE/SPIRE service identities.

## Secrets Management

Never commit secrets. Use a secrets manager:

- HashiCorp Vault (dynamic secrets, audit).
- AWS KMS / GCP KMS / Azure Key Vault.
- SOPS + age for git-encrypted secrets at rest.

Inject secrets at runtime via sidecars, mounted volumes, or External Secrets
Operator — not as plain env vars in source.

## Observability

Security and reliability require visibility. Use OpenTelemetry to emit
metrics, logs, and traces to a backend of choice (Prometheus, Jaeger, Datadog,
Honeycomb). Instrument from day one.

## Decision Checklist

1. Is traffic over the public internet? TLS 1.3 minimum.
2. Is it service-to-service? Use mTLS or SPIFFE identities.
3. Are users authenticating? Use OIDC with a trusted identity provider.
4. Are there third-party API calls? Use scoped API keys, rotate regularly.
5. Can an attacker pivot on a leaked credential? Enforce least privilege and
   short-lived tokens.

## See Also

- [Load Balancing and Reverse Proxy](load-balancing-and-proxy.md) — SSL
  termination, mTLS via service mesh.
- [Authentication and Environment Management](auth-env.md) — environment
  detection for auth path selection.
- [Data Access Layer](data-access-layer.md) — the DAL as the authorization
  checkpoint.

## Sources

- [The System Design Primer](https://github.com/donnemartin/system-design-primer)
  — Security (encrypt, sanitize, parameterized queries, least privilege).
- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [OpenTelemetry](https://opentelemetry.io/)
