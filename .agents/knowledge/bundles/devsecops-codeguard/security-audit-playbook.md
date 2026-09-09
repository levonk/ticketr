---
type: Practice
title: Security Audit Playbook — Final Validation for Cloud Server Deployments
description: Run a final security audit playbook that checks SSH, firewall, fail2ban, Docker daemon hardening, automatic updates, and container image age before considering a deployment complete.
tags: [devsecops, security, audit, ansible, cloud, ssh, firewall, docker]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: infrahub-final-audit-yml
    resource: "https://github.com/levonk/infrahub/blob/main/shared/active/02-config/ansible/playbooks/final-audit.yml"
    title: "infrahub final-audit.yml"
  - id: infrahub-agents-md-security-audit-guidelines
    resource: "https://github.com/levonk/infrahub/blob/main/AGENTS.md"
    title: "infrahub AGENTS.md Security Audit Guidelines"
---


# Security Audit Playbook

## Failure Mode

Cloud servers are deployed with hardened playbooks but never validated, so
drift, forgotten steps, or configuration mistakes leave security gaps.

## Symptoms

- A server ships without `fail2ban` running.
- Docker daemon runs without `userns-remap` or `no-new-privileges`.
- Automatic security updates are not configured.
- Container images are >30 days old and contain unpatched CVEs.
- Hardcoded IPs/ports appear in deployed configs.

## Practice

### Playbook Structure

```yaml
- name: "Final Security Audit"
  hosts: cloud_servers
  become: true
  gather_facts: true
  vars:
    audit_results: {}
    security_gaps: []
    security_warnings: []
```

### Critical Checks

| Check | Why |
|-------|-----|
| SSH connectivity | Verify the server is reachable after hardening |
| No hardcoded IPs/ports | Enforce infrastructure consolidation variables |
| SSH hardening | `PermitRootLogin no/prohibit-password`, `PasswordAuthentication no`, ed25519-only |
| Firewall default-deny | Only explicitly allowed ports are open |
| fail2ban | Brute-force protection active |
| Docker daemon hardening | `userns-remap` or `no-new-privileges` |
| Automatic updates | `dnf-automatic` (RedHat) or `unattended-upgrades` (Debian) |

### Non-Critical Warnings

- Container image age >30 days should trigger a warning, not a failure.
- Use a separate `security_warnings` list and surface it at the end of the
  playbook output.

### Remediation

- Failing a critical check should block deployment.
- Warnings should be tracked and remediated in a follow-up task.
- Audit results should be logged and optionally published to a security
  dashboard.
