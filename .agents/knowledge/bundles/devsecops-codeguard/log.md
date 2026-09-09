# Directory Update Log

## 2026-08-27

* **Update**: Updated [dependency-supply-chain.md](dependency-supply-chain.md)
  to add a "Workflow Supply Chain (zizmor + actionlint)" section documenting
  GitHub Actions workflows as a dependency surface. Covers zizmor (unpinned
  actions, excessive permissions, OIDC misuse) and actionlint (syntax, schema,
  deprecated syntax) as the standard workflow security linting tools. Added
  cross-reference to
  [pre-commit-ci-parity.md](https://github.com/levonk/skills-releases/blob/main/knowledge/cicd-testing-practices/pre-commit-ci-parity.md)
  in the cicd-testing-practices bundle. Updated frontmatter with zizmor and
  actionlint sources.

## 2026-08-05

* **Ingest**: Authored 3 new concept pages sourced from the project-lint
  audit. Includes one TODO promotion (dependency-supply-chain) and one
  cross-bundle synthesis page (security-aware-static-analysis). All pages
  grounded against current tool versions on 2026-08-05.
  - [linter-security-patterns.md](linter-security-patterns.md) — security of
    linter output, untrusted code handling, sandboxing AST parsing, plugin
    security, telemetry minimization
  - [security-aware-static-analysis.md](security-aware-static-analysis.md) —
    cross-bundle synthesis: untrusted code, info leakage, sandboxing,
    supply-chain security for analysis tools
  - [dependency-supply-chain.md](dependency-supply-chain.md) — lockfile
    pinning, cargo-deny/cargo-audit, private registries, SLSA provenance
    (promoted from overview TODO)
* **Update**: Updated [index.md](index.md) with 3 new concept entries.
* **Update**: Updated [overview.md](overview.md.tmpl) — promoted
  `dependency-supply-chain.md` from "Future concept candidates" TODO to a real
  page; annotated remaining 4 TODOs with deferral rationale.

## 2026-07-26
* **Migration**: Migrated bundle from OKF v0.1 to OKF v0.2 — bumped `okf_version` in index.md. No `# Citations` sections or `timestamp` fields to migrate.
* **Migration**: Migrated `## Citations` body sections to `sources` frontmatter with stable `id` attributes per OKF v0.2 §13.1.

## 2026-07-17

* **Restructure**: Moved container-specific concepts to the
  [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/index.md) bundle,
  which is the canonical home for container authoring, runtime, and Dockerfile
  practices. Moved files:
  - [container-hardening.md](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/container-runtime-hardening.md) → merged into `container-runtime-hardening.md` (unique sections: docker.sock prohibition, TCP daemon, image scanning, implementation checklist)
  - [nodejs-in-containers.md](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/nodejs-in-containers.md) → moved as-is
  - [dockerfile-best-practices.md](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/dockerfile-best-practices.md) → moved as-is
  Updated [overview.md](overview.md) pipeline, scope, and citations to reflect
  the narrower code-level security + audit focus. Updated [index.md](index.md)
  to remove the three container concept entries.

## 2026-07-17

* **Update**: Added missing concept pages referenced by [index.md](index.md):
  - [ssh-hardening.md](ssh-hardening.md) — PermitRootLogin, ed25519-only keys, fail2ban
  - [security-audit-playbook.md](security-audit-playbook.md) — final-audit.yml validation pattern
  - [dockerfile-best-practices.md](dockerfile-best-practices.md) — apt/apk cleanup, user creation, multi-stage builds, HEALTHCHECK (since moved to container-best-practices)

## 2026-07-17

* **Initialization**: Created the `devsecops-codeguard` knowledge bundle to consolidate codeguard rules from job-aide and security audit practices from infrahub into a generalizable OKF v0.1 knowledge base.
* **Creation**: Authored 7 concept pages covering the DevSecOps codeguard spectrum — from C memory safety and crypto algorithm governance through SSH hardening and infrastructure security audits. (Originally 10 pages; 3 container-specific pages were later moved to container-best-practices.)
  - [safe-c-functions.md](safe-c-functions.md) — banned memory/string functions, bounded replacements, compiler flags
  - [hardcoded-credentials-detection.md](hardcoded-credentials-detection.md) — AWS/Stripe/Google/GitHub/JWT patterns, private keys, connection strings
  - [crypto-algorithm-governance.md](crypto-algorithm-governance.md) — banned algorithms, deprecated OpenSSL APIs, EVP replacements
  - [digital-certificate-validation.md](digital-certificate-validation.md) — expiration, key strength, signature algorithm, self-signed checks
  - [ssh-hardening.md](ssh-hardening.md) — PermitRootLogin, ed25519-only, PasswordAuthentication no, fail2ban
  - [security-audit-playbook.md](security-audit-playbook.md) — final-audit.yml pattern, hardcoded IP/port checks, Docker daemon hardening
* **Creation**: Established [overview.md](overview.md) synthesis and [index.md](index.md) directory listing.
