---
okf_version: "0.2"
---

# DevSecOps Codeguard

A compounding knowledge base documenting DevSecOps codeguard practices — the
rules that prevent the most common security defects from reaching production.
Each concept captures a specific class of vulnerability or misconfiguration
and the practice that eliminates it, sourced from real codeguard rules and
security audit playbooks. Container-specific hardening and Dockerfile practices
live in the [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/index.md)
bundle.

## Concepts

* [Overview](overview.md) - Synthesis of the full DevSecOps codeguard practice set and how the pieces fit together
* [Safe C Functions](safe-c-functions.md) - Banned memory/string functions (memcpy, strcpy, sprintf) and their bounded replacements, plus compiler hardening flags
* [Hardcoded Credentials Detection](hardcoded-credentials-detection.md) - Recognition patterns for AWS, Stripe, Google, GitHub, JWT keys, private key blocks, and connection strings
* [Crypto Algorithm Governance](crypto-algorithm-governance.md) - Banned algorithms (MD5, SHA-1, RC4, DES), deprecated OpenSSL APIs, and secure replacements via EVP high-level APIs
* [Digital Certificate Validation](digital-certificate-validation.md) - Expiration, key strength, signature algorithm, and self-signed checks for X.509 certificates
* [SSH Hardening](ssh-hardening.md) - PermitRootLogin settings, ed25519-only keys, PasswordAuthentication no, fail2ban
* [Security Audit Playbook](security-audit-playbook.md) - The final-audit.yml pattern: SSH checks, no hardcoded IPs/ports, firewall default-deny, fail2ban, Docker daemon hardening, automatic updates, container image age
* [Linter Security Patterns](linter-security-patterns.md) - Security of linter output, safe handling of untrusted code, sandboxing AST parsing, plugin security, telemetry minimization
* [Security-Aware Static Analysis](security-aware-static-analysis.md) - Cross-bundle synthesis: untrusted code handling, info leakage prevention, sandboxing, supply-chain security for analysis tools
* [Dependency Supply Chain](dependency-supply-chain.md) - Lockfile pinning, cargo-deny/cargo-audit, private registries, SLSA provenance, PR-time dependency review
