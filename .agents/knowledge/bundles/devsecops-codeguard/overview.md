---
type: Synthesis
title: DevSecOps Codeguard Overview
description: Synthesis of DevSecOps codeguard practices — C memory safety, credential detection, crypto governance, certificate validation, SSH hardening, and infrastructure security audits. Container-specific hardening and Dockerfile practices live in the container-best-practices bundle.
tags: [devsecops, security, codeguard, overview, synthesis]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: codeguard-1-safe-c-functions
    resource: ".devin/rules/codeguard-1-safe-c-functions.md"
    title: "job-aide"
  - id: codeguard-1-hardcoded-credentials
    resource: ".devin/rules/codeguard-1-hardcoded-credentials.md"
    title: "job-aide"
  - id: codeguard-1-crypto-algorithms
    resource: ".devin/rules/codeguard-1-crypto-algorithms.md"
    title: "job-aide"
  - id: codeguard-1-digital-certificates
    resource: ".devin/rules/codeguard-1-digital-certificates.md"
    title: "job-aide"
  - id: final-audit
    resource: "shared/active/02-config/ansible/playbooks/final-audit.yml"
    title: "infrahub"
  - id: agents
    resource: "AGENTS.md"
    title: "Security Audit Guidelines — infrahub"
---

---
description: STE100-inspired Simplified Technical English guidelines for technical prose output — active voice, short sentences, one-word-one-meaning, imperative for instructions
---

### Simplified Technical English (STE100-Inspired)

This artifact produces technical English (instructions, procedures, descriptions,
reference documentation). Apply these STE100-inspired guidelines to all technical
prose output so the result is unambiguous, translatable, and easy to read for
non-native speakers and for AI agents that must execute the steps precisely.

These are **STE-inspired guidelines**, not the full ASD-STE100 vocabulary
restriction. Domain terms (Dockerfile, pnpm, devbox, Nx, etc.) are permitted
when they are the correct technical term — STE100's 1000-word approved
vocabulary is too narrow for this domain. The goal is the *clarity discipline*
of STE100, not its word list.

For the full writing rules, before/after examples, and the approved-words
reference, see the
[Simplified Technical English](https://github.com/levonk/skills-releases/blob/main/knowledge/simplified-technical-english/simplified-technical-english.md)
and
[Detailed Guide](https://github.com/levonk/skills-releases/blob/main/knowledge/simplified-technical-english/detailed-guide.md)
concept pages in the `simplified-technical-english` knowledge bundle. The
bundle is the canonical, publicly-reachable home for these guidelines; this
include is the build-time gist that gets inlined into skills and other
bundles.

#### Core Principles

1. **One word, one meaning.** Pick one term for each concept and use it
   everywhere. Do not alternate between "image" and "container image" and
   "docker image" for the same thing. Pick one, define it once, reuse it.

2. **Short sentences.** Keep procedural sentences under 20 words. Keep
   descriptive sentences under 25 words. Split long sentences into two.

3. **Active voice.** Write "The build copies the file" not "The file is copied
   by the build." The actor does the action. Passive voice hides who does what
   and is the single largest source of ambiguity in technical prose.

4. **Imperative mood for instructions.** Write "Run the tests" not "You should
   run the tests" or "The tests should be run." Instructions tell the reader
   (or agent) what to do, directly.

5. **One topic per sentence.** One idea per sentence. Do not chain unrelated
   clauses with "and" or "while." If a sentence has two ideas, split it.

6. **Consistent verb forms.** Use the same verb for the same action across the
   document. If you "run" a script in section 1, do not "execute" it in
   section 2. Pick one verb per action and keep it.

7. **Approved modifiers only.** Avoid decorative adjectives and adverbs
   ("very", "extremely", "simply", "just"). Keep modifiers that carry
   information ("non-root", "read-only", "idempotent"). Drop modifiers that
   carry only emphasis.

8. **Define every acronym on first use.** Write "Continuous Integration (CI)"
   on first use, then "CI" thereafter. Never assume the reader knows the
   acronym.

#### What Counts as Technical English

Apply these guidelines to:

- Procedural instructions ("Run `just build`", "Add the user to the group")
- Descriptions of failure modes, symptoms, and practices
- Reference documentation and concept pages
- Checklists and review guidance
- Synthesis and overview prose in knowledge bundles

Do **not** apply these guidelines to:

- Code, commands, and file paths (those have their own syntax)
- Frontmatter and structured data (YAML, JSON)
- Diagrams and their source syntax (Mermaid, PlantUML)
- Business communication, marketing copy, or creative content
- Log entries and change logs (those are append-only records)

#### Quick Self-Check

Before finishing a piece of technical prose, run this checklist:

- [ ] Is every sentence under 25 words? (Procedural: under 20.)
- [ ] Is every sentence active voice? (Or is the passive voice intentional and
      necessary?)
- [ ] Are instructions in imperative mood?
- [ ] Does each technical term have one and only one form in this document?
- [ ] Is every acronym defined on first use?
- [ ] Are decorative modifiers removed?
- [ ] Does each sentence carry one topic?

If any answer is "no," revise before publishing.



# DevSecOps Codeguard Overview

This bundle documents practices that prevent the most common security defects
from reaching production. Each concept was extracted from real codeguard rules
and security audit playbooks — banned C functions that cause buffer overflows,
hardcoded credentials that leak secrets, deprecated crypto algorithms that
break confidentiality, misconfigured certificates that fail TLS, and SSH
daemons that accept weak authentication. Container-specific hardening and
Dockerfile practices have been moved to the
[container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md) bundle,
which covers runtime hardening, Node.js in containers, and Dockerfile best
practices in greater depth.

## The DevSecOps Pipeline

```
source-code → crypto → certificates → credentials → runtime → audit
```

Each phase has practices that prevent specific failure modes:

| Phase | Practice | Prevents |
|-------|----------|----------|
| Source code | [Safe C Functions](safe-c-functions.md) | Buffer overflows from unbounded memory/string operations |
| Crypto | [Crypto Algorithm Governance](crypto-algorithm-governance.md) | Broken confidentiality from MD5, SHA-1, RC4, DES, deprecated OpenSSL APIs |
| Certificates | [Digital Certificate Validation](digital-certificate-validation.md) | TLS failures from expired certs, weak keys, SHA-1 signatures, self-signed certs in prod |
| Secrets | [Hardcoded Credentials Detection](hardcoded-credentials-detection.md) | Credential leaks from AWS/Stripe/Google/GitHub/JWT keys committed to source |
| Runtime | [SSH Hardening](ssh-hardening.md) | Brute-force attacks, root login via password, weak key types |
| Audit | [Security Audit Playbook](security-audit-playbook.md) | Drift from hardening baseline, stale container images, unenforced firewall |
| Linter Sec | [Linter Security Patterns](linter-security-patterns.md) | Linter leaking secrets in errors, untrusted code execution, plugin privilege escalation |
| Analysis Sec | [Security-Aware Static Analysis](security-aware-static-analysis.md) | Cross-bundle: untrusted code handling, info leakage, sandboxing, tool supply chain |
| Deps | [Dependency Supply Chain](dependency-supply-chain.md) | Unpinned dependencies, missing advisory checks, no SLSA provenance |

## Scope

This bundle covers **code-level security and runtime security audits** — the
practices that ship in source code, SSH configurations, and Ansible audit
playbooks. Container-specific hardening (runtime controls, Dockerfile
patterns, Node.js in containers) lives in the
[container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md) bundle. It
does **not** cover:

- Container runtime hardening, Dockerfile best practices, or Node.js container
  production hardening — see
  [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md).
- Application-level authentication and authorization patterns (OAuth flows,
  RBAC design) — separate bundle.
- Network security architecture (zero-trust, segmentation) — separate bundle.
- Security monitoring and incident response (SIEM, SOAR) — separate bundle.
- Cloud provider security services (GuardDuty, Security Hub) — separate bundle.

## Relationship to Source Rules

The codeguard rules in job-aide (`.devin/rules/codeguard-*.md`) are
always-on AI agent rules that enforce these practices during code generation
and review. This bundle provides the **generalizable knowledge** behind those
rules — the "why" that explains what each rule prevents and how to apply it
correctly in context.

The infrahub `final-audit.yml` playbook operationalizes these practices as
post-deployment validation. This bundle's [Security Audit Playbook](security-audit-playbook.md)
captures the pattern so it can be reapplied to new environments.

## Sources

The initial concepts were extracted from:

- job-aide `.devin/rules/` codeguard rules (safe C functions, hardcoded
  credentials, crypto algorithms, digital certificates)
- infrahub `shared/active/02-config/ansible/playbooks/final-audit.yml` security
  audit playbook (SSH hardening, firewall, fail2ban, Docker daemon hardening)
- infrahub `AGENTS.md` security audit guidelines and SSH hardening best
  practices

See each concept's `# Citations` section for the specific source files.

## Compounding

New lessons from future security work — CVE post-mortems, audit findings,
new tooling, or new attack vectors — should be filed as new concept pages.
The trigger for adding a concept is: a security incident, a failed audit, a
new CVE that reveals a practice the bundle doesn't yet cover, or a new
codeguard rule added to job-aide. Append to `log.md` when adding.

Future concept candidates (not yet in the bundle):

- `secrets-management.md` — vault patterns, KMS integration, runtime secret
  retrieval vs. baked-in credentials — deferred: out of scope for current
  cycle (covered by secrets-egress-security bundle)
- `ci-cd-pipeline-security.md` — protected branches, signed commits,
  ephemeral runners, security gates (SAST/SCA/DAST) — deferred: out of scope
  for current cycle (no CI pipeline security incident yet)
- `virtual-patching.md` — WAF/IPS/ModSecurity for temporary CVE mitigation —
  deferred: out of scope for current cycle (no WAF infrastructure)
- `c-toolchain-hardening.md` — compiler flags (PIE, RELRO, CFI), linker
  hardening, checksec verification in CI — deferred: out of scope for current
  cycle (no C/C++ projects in active use)

Promoted from TODO on 2026-08-05:

- `dependency-supply-chain.md` — promoted to a real page; see
  [Dependency Supply Chain](dependency-supply-chain.md)

## Related Knowledge Bundles

- [container-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/container-best-practices/overview.md) —
  Dockerfile best practices and container runtime hardening that complement
  the codeguard container rules.
- [typescript-monorepo-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/typescript-monorepo-best-practices/overview.md)
  — TypeScript project conventions that interact with credential detection,
  crypto usage, and ESLint security rules.
- [java-best-practices](https://github.com/levonk/skills-releases/blob/main/knowledge/java-best-practices/overview.md) — Java security
  practices (dependency scanning, SAST, JEP 411) that complement codeguard rules.

---

## Content Ordering

This artifact is optimized for machine consumption. Generic framework content
(shared includes, knowledge bundles) appears before the skill-specific body.
This ordering maximizes cross-skill prefix caching: skills that share the same
includes produce identical byte prefixes, so an LLM context cache warmed by one
skill serves all skills that share the same preamble.

This is sub-optimal for human reading — the skill-specific content starts deep
in the file, after the generic preamble. Human readers can jump to the
skill-specific body by searching for the first `# ` heading that follows the
generic sections. Each section is self-contained and documented with its own
heading hierarchy.

